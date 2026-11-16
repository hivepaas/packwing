package com.hivepaas.packwing

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import android.util.Log
import uniffi.packwing_mobile.MobileException
import uniffi.packwing_mobile.Vault
import java.security.KeyStore
import javax.crypto.AEADBadTagException
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * Where a storage's secrets and the keys kept are: Android's keychain, as the
 * system's holds them on the desktop and on iOS (crates/mobile/src/vault.rs).
 *
 * Each secret is encrypted with AES-GCM by a key of the Android Keystore,
 * made the first time one is kept, which never leaves it; what is kept in the
 * app's preferences is the nonce and the encrypted secret, and nothing that
 * opens it. Those preferences are left out of backups (res/xml), as the key
 * could never come with them: one that cannot be opened is taken as none, and
 * the user types it again.
 *
 * The core calls it from whatever thread its work is on.
 */
class KeystoreVault(context: Context) : Vault {
    private val prefs = context.getSharedPreferences("vault", Context.MODE_PRIVATE)

    @Synchronized
    override fun get(service: String, account: String): ByteArray? = guarded {
        val kept = prefs.getString(name(service, account), null) ?: return@guarded null
        val bytes = Base64.decode(kept, Base64.NO_WRAP)
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(TAG_BITS, bytes, 0, NONCE_BYTES))
        try {
            cipher.doFinal(bytes, NONCE_BYTES, bytes.size - NONCE_BYTES)
        } catch (e: AEADBadTagException) {
            // Sealed by a key there is no more of: as good as none.
            Log.w("packwing", "a secret kept no key opens; forgotten", e)
            prefs.edit().remove(name(service, account)).commit()
            null
        }
    }

    @Synchronized
    override fun set(service: String, account: String, secret: ByteArray) = guarded {
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, key())
        val sealed = cipher.iv + cipher.doFinal(secret)
        // Written before it returns: a secret said to be kept is.
        check(prefs.edit().putString(name(service, account), Base64.encodeToString(sealed, Base64.NO_WRAP)).commit())
    }

    @Synchronized
    override fun delete(service: String, account: String): Boolean = guarded {
        val name = name(service, account)
        val was = prefs.contains(name)
        prefs.edit().remove(name).commit()
        was
    }

    /** The Keystore's key, made the first time a secret is kept. */
    private fun key(): SecretKey {
        val store = KeyStore.getInstance(KEYSTORE).apply { load(null) }
        (store.getKey(ALIAS, null) as? SecretKey)?.let { return it }
        val made = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, KEYSTORE)
        made.init(
            KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build(),
        )
        return made.generateKey()
    }

    /** What went wrong, as the core takes it: a failure it says, never a crash. */
    private fun <T> guarded(work: () -> T): T = try {
        work()
    } catch (e: Exception) {
        Log.w("packwing", "the vault failed", e)
        throw MobileException.Failed(tr("packwing could not reach its keychain."))
    }

    private companion object {
        const val KEYSTORE = "AndroidKeyStore"
        const val ALIAS = "packwing-vault"
        const val TRANSFORMATION = "AES/GCM/NoPadding"
        const val NONCE_BYTES = 12
        const val TAG_BITS = 128

        /** A secret's name in the preferences: its service and account, apart. */
        fun name(service: String, account: String) = "$service\u0000$account"
    }
}
