package com.gamshik.securestore

import android.app.Activity
import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

@InvokeArg
class KeyArgs {
    lateinit var key: String
}

@InvokeArg
class SetArgs {
    lateinit var key: String
    lateinit var value: String
}

/**
 * Secrets are encrypted with AES-256-GCM. The AES key is generated inside the Android Keystore
 * and never leaves it; only the ciphertext ("<iv>:<ciphertext>", Base64) is written to this
 * app's private SharedPreferences. The entry name is bound as associated data, so a ciphertext
 * can't be swapped between entries.
 */
@TauriPlugin
class SecureStorePlugin(private val activity: Activity) : Plugin(activity) {
    private val prefs = activity.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    private fun secretKey(): SecretKey {
        val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
        (keyStore.getEntry(KEY_ALIAS, null) as? KeyStore.SecretKeyEntry)?.let { return it.secretKey }

        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
        generator.init(
            KeyGenParameterSpec.Builder(
                KEY_ALIAS,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
            )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build(),
        )
        return generator.generateKey()
    }

    private fun encrypt(name: String, plain: String): String {
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, secretKey())
        cipher.updateAAD(name.toByteArray(Charsets.UTF_8))
        val ct = cipher.doFinal(plain.toByteArray(Charsets.UTF_8))
        return b64(cipher.iv) + ":" + b64(ct)
    }

    private fun decrypt(name: String, stored: String): String {
        val (iv, ct) = stored.split(":", limit = 2).map { Base64.decode(it, Base64.NO_WRAP) }
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.DECRYPT_MODE, secretKey(), GCMParameterSpec(128, iv))
        cipher.updateAAD(name.toByteArray(Charsets.UTF_8))
        return String(cipher.doFinal(ct), Charsets.UTF_8)
    }

    @Command
    fun get(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(KeyArgs::class.java)
            val stored = prefs.getString(args.key, null)
            val value = stored?.let {
                try {
                    decrypt(args.key, it)
                } catch (ex: Exception) {
                    // Happens when the Keystore key is gone (e.g. data restored from a backup onto
                    // another device). The ciphertext is useless then, so drop it and report "not set".
                    Log.w(TAG, "Dropping unreadable entry '${args.key}'", ex)
                    prefs.edit().remove(args.key).apply()
                    null
                }
            }
            invoke.resolve(JSObject().put("value", value))
        } catch (ex: Exception) {
            invoke.reject(ex.message ?: "Failed to read from secure storage")
        }
    }

    @Command
    fun set(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SetArgs::class.java)
            val ok = prefs.edit().putString(args.key, encrypt(args.key, args.value)).commit()
            if (ok) invoke.resolve() else invoke.reject("Failed to write to secure storage")
        } catch (ex: Exception) {
            invoke.reject(ex.message ?: "Failed to write to secure storage")
        }
    }

    @Command
    fun delete(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(KeyArgs::class.java)
            prefs.edit().remove(args.key).commit()
            invoke.resolve()
        } catch (ex: Exception) {
            invoke.reject(ex.message ?: "Failed to delete from secure storage")
        }
    }

    private fun b64(bytes: ByteArray) = Base64.encodeToString(bytes, Base64.NO_WRAP)

    companion object {
        private const val TAG = "SecureStore"
        private const val PREFS = "secure_store"
        private const val ANDROID_KEYSTORE = "AndroidKeyStore"
        private const val KEY_ALIAS = "secure_store_master_key"
        private const val TRANSFORMATION = "AES/GCM/NoPadding"
    }
}
