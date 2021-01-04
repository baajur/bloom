use error::ErrorStack;
use ffi;
use foreign_types::{ForeignType, ForeignTypeRef};
use std::mem;
use std::ptr;

use bn::{BigNum, BigNumRef};
use pkey::{HasParams, HasPrivate, HasPublic, Params, Private};
use {cvt, cvt_p};

generic_foreign_type_and_impl_send_sync! {
    type CType = ffi::DH;
    fn drop = ffi::DH_free;

    pub struct Dh<T>;

    pub struct DhRef<T>;
}

impl<T> DhRef<T>
where
    T: HasParams,
{
    to_pem! {
        /// Serializes the parameters into a PEM-encoded PKCS#3 DHparameter structure.
        ///
        /// The output will have a header of `-----BEGIN DH PARAMETERS-----`.
        ///
        /// This corresponds to [`PEM_write_bio_DHparams`].
        ///
        /// [`PEM_write_bio_DHparams`]: https://www.openssl.org/docs/manmaster/man3/PEM_write_bio_DHparams.html
        params_to_pem,
        ffi::PEM_write_bio_DHparams
    }

    to_der! {
        /// Serializes the parameters into a DER-encoded PKCS#3 DHparameter structure.
        ///
        /// This corresponds to [`i2d_DHparams`].
        ///
        /// [`i2d_DHparams`]: https://www.openssl.org/docs/man1.1.0/crypto/i2d_DHparams.html
        params_to_der,
        ffi::i2d_DHparams
    }
}

impl Dh<Params> {
    pub fn from_params(p: BigNum, g: BigNum, q: BigNum) -> Result<Dh<Params>, ErrorStack> {
        Self::from_pqg(p, Some(q), g)
    }

    /// Creates a DH instance based upon the given primes and generator params.
    ///
    /// This corresponds to [`DH_new`] and [`DH_set0_pqg`].
    ///
    /// [`DH_new`]: https://www.openssl.org/docs/man1.1.0/crypto/DH_new.html
    /// [`DH_set0_pqg`]: https://www.openssl.org/docs/man1.1.0/crypto/DH_set0_pqg.html
    pub fn from_pqg(
        prime_p: BigNum,
        prime_q: Option<BigNum>,
        generator: BigNum,
    ) -> Result<Dh<Params>, ErrorStack> {
        unsafe {
            let dh = Dh::from_ptr(cvt_p(ffi::DH_new())?);
            cvt(DH_set0_pqg(
                dh.0,
                prime_p.as_ptr(),
                prime_q.as_ref().map_or(ptr::null_mut(), |q| q.as_ptr()),
                generator.as_ptr(),
            ))?;
            mem::forget((prime_p, prime_q, generator));
            Ok(dh)
        }
    }

    /// Generates DH params based on the given `prime_len` and a fixed `generator` value.
    ///
    /// This corresponds to [`DH_generate_parameters`].
    ///
    /// [`DH_generate_parameters`]: https://www.openssl.org/docs/man1.1.0/crypto/DH_generate_parameters.html
    pub fn generate_params(prime_len: u32, generator: u32) -> Result<Dh<Params>, ErrorStack> {
        unsafe {
            Ok(Dh::from_ptr(cvt_p(ffi::DH_generate_parameters(
                prime_len as i32,
                generator as i32,
                None,
                ptr::null_mut(),
            ))?))
        }
    }

    /// Generates a public and a private key based on the DH params.
    ///
    /// This corresponds to [`DH_generate_key`].
    ///
    /// [`DH_generate_key`]: https://www.openssl.org/docs/man1.1.0/crypto/DH_generate_key.html
    pub fn generate_key(self) -> Result<Dh<Private>, ErrorStack> {
        unsafe {
            let dh_ptr = self.0;
            cvt(ffi::DH_generate_key(dh_ptr))?;
            mem::forget(self);
            Ok(Dh::from_ptr(dh_ptr))
        }
    }

    from_pem! {
        /// Deserializes a PEM-encoded PKCS#3 DHpararameters structure.
        ///
        /// The input should have a header of `-----BEGIN DH PARAMETERS-----`.
        ///
        /// This corresponds to [`PEM_read_bio_DHparams`].
        ///
        /// [`PEM_read_bio_DHparams`]: https://www.openssl.org/docs/man1.0.2/crypto/PEM_read_bio_DHparams.html
        params_from_pem,
        Dh<Params>,
        ffi::PEM_read_bio_DHparams
    }

    from_der! {
        /// Deserializes a DER-encoded PKCS#3 DHparameters structure.
        ///
        /// This corresponds to [`d2i_DHparams`].
        ///
        /// [`d2i_DHparams`]: https://www.openssl.org/docs/man1.1.0/crypto/d2i_DHparams.html
        params_from_der,
        Dh<Params>,
        ffi::d2i_DHparams
    }

    /// Requires OpenSSL 1.0.2 or newer.
    #[cfg(any(ossl102, ossl110))]
    pub fn get_1024_160() -> Result<Dh<Params>, ErrorStack> {
        unsafe {
            ffi::init();
            cvt_p(ffi::DH_get_1024_160()).map(|p| Dh::from_ptr(p))
        }
    }

    /// Requires OpenSSL 1.0.2 or newer.
    #[cfg(any(ossl102, ossl110))]
    pub fn get_2048_224() -> Result<Dh<Params>, ErrorStack> {
        unsafe {
            ffi::init();
            cvt_p(ffi::DH_get_2048_224()).map(|p| Dh::from_ptr(p))
        }
    }

    /// Requires OpenSSL 1.0.2 or newer.
    #[cfg(any(ossl102, ossl110))]
    pub fn get_2048_256() -> Result<Dh<Params>, ErrorStack> {
        unsafe {
            ffi::init();
            cvt_p(ffi::DH_get_2048_256()).map(|p| Dh::from_ptr(p))
        }
    }
}

impl<T> Dh<T>
where
    T: HasParams,
{
    /// Returns the prime `p` from the DH instance.
    ///
    /// This corresponds to [`DH_get0_pqg`].
    ///
    /// [`DH_get0_pqg`]: https://www.openssl.org/docs/man1.1.0/crypto/DH_get0_pqg.html
    pub fn prime_p(&self) -> &BigNumRef {
        let mut p = ptr::null();
        unsafe {
            DH_get0_pqg(self.as_ptr(), &mut p, ptr::null_mut(), ptr::null_mut());
            BigNumRef::from_ptr(p as *mut _)
        }
    }

    /// Returns the prime `q` from the DH instance.
    ///
    /// This corresponds to [`DH_get0_pqg`].
    ///
    /// [`DH_get0_pqg`]: https://www.openssl.org/docs/man1.1.0/crypto/DH_get0_pqg.html
    pub fn prime_q(&self) -> Option<&BigNumRef> {
        let mut q = ptr::null();
        unsafe {
            DH_get0_pqg(self.as_ptr(), ptr::null_mut(), &mut q, ptr::null_mut());
            if q.is_null() {
                None
            } else {
                Some(BigNumRef::from_ptr(q as *mut _))
            }
        }
    }

    /// Returns the generator from the DH instance.
    ///
    /// This corresponds to [`DH_get0_pqg`].
    ///
    /// [`DH_get0_pqg`]: https://www.openssl.org/docs/man1.1.0/crypto/DH_get0_pqg.html
    pub fn generator(&self) -> &BigNumRef {
        let mut g = ptr::null();
        unsafe {
            DH_get0_pqg(self.as_ptr(), ptr::null_mut(), ptr::null_mut(), &mut g);
            BigNumRef::from_ptr(g as *mut _)
        }
    }
}

impl<T> DhRef<T>
where
    T: HasPublic,
{
    /// Returns the public key from the DH instance.
    ///
    /// This corresponds to [`DH_get0_key`].
    ///
    /// [`DH_get0_key`]: https://www.openssl.org/docs/man1.1.0/crypto/DH_get0_key.html
    pub fn public_key(&self) -> &BigNumRef {
        let mut pub_key = ptr::null();
        unsafe {
            DH_get0_key(self.as_ptr(), &mut pub_key, ptr::null_mut());
            BigNumRef::from_ptr(pub_key as *mut _)
        }
    }
}

impl<T> DhRef<T>
where
    T: HasPrivate,
{
    /// Computes a shared secret from the own private key and the given `public_key`.
    ///
    /// This corresponds to [`DH_compute_key`].
    ///
    /// [`DH_compute_key`]: https://www.openssl.org/docs/man1.1.0/crypto/DH_compute_key.html
    pub fn compute_key(&self, public_key: &BigNumRef) -> Result<Vec<u8>, ErrorStack> {
        unsafe {
            let key_len = ffi::DH_size(self.as_ptr());
            let mut key = vec![0u8; key_len as usize];
            cvt(ffi::DH_compute_key(
                key.as_mut_ptr(),
                public_key.as_ptr(),
                self.as_ptr(),
            ))?;
            Ok(key)
        }
    }
}

cfg_if! {
    if #[cfg(any(ossl110, libressl270))] {
        use ffi::{DH_set0_pqg, DH_get0_pqg, DH_get0_key};
    } else {
        #[allow(bad_style)]
        unsafe fn DH_set0_pqg(
            dh: *mut ffi::DH,
            p: *mut ffi::BIGNUM,
            q: *mut ffi::BIGNUM,
            g: *mut ffi::BIGNUM,
        ) -> ::libc::c_int {
            (*dh).p = p;
            (*dh).q = q;
            (*dh).g = g;
            1
        }

        #[allow(bad_style)]
        unsafe fn DH_get0_pqg(
            dh: *mut ffi::DH,
            p: *mut *const ffi::BIGNUM,
            q: *mut *const ffi::BIGNUM,
            g: *mut *const ffi::BIGNUM,
        ) {
            if !p.is_null() {
                *p = (*dh).p;
            }
            if !q.is_null() {
                *q = (*dh).q;
            }
            if !g.is_null() {
                *g = (*dh).g;
            }
        }

        #[allow(bad_style)]
        unsafe fn DH_get0_key(
            dh: *mut ffi::DH,
            pub_key: *mut *const ffi::BIGNUM,
            priv_key: *mut *const ffi::BIGNUM,
        ) {
            if !pub_key.is_null() {
                *pub_key = (*dh).pub_key;
            }
            if !priv_key.is_null() {
                *priv_key = (*dh).priv_key;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bn::BigNum;
    use dh::Dh;
    use ssl::{SslContext, SslMethod};

    #[test]
    #[cfg(ossl102)]
    fn test_dh_rfc5114() {
        let mut ctx = SslContext::builder(SslMethod::tls()).unwrap();
        let dh2 = Dh::get_2048_224().unwrap();
        ctx.set_tmp_dh(&dh2).unwrap();
        let dh3 = Dh::get_2048_256().unwrap();
        ctx.set_tmp_dh(&dh3).unwrap();
    }

    #[test]
    fn test_dh_params() {
        let mut ctx = SslContext::builder(SslMethod::tls()).unwrap();
        let prime_p = BigNum::from_hex_str(
            "87A8E61DB4B6663CFFBBD19C651959998CEEF608660DD0F25D2CEED4435E3B00E00DF8F1D61957D4FAF7DF\
             4561B2AA3016C3D91134096FAA3BF4296D830E9A7C209E0C6497517ABD5A8A9D306BCF67ED91F9E6725B47\
             58C022E0B1EF4275BF7B6C5BFC11D45F9088B941F54EB1E59BB8BC39A0BF12307F5C4FDB70C581B23F76B6\
             3ACAE1CAA6B7902D52526735488A0EF13C6D9A51BFA4AB3AD8347796524D8EF6A167B5A41825D967E144E5\
             140564251CCACB83E6B486F6B3CA3F7971506026C0B857F689962856DED4010ABD0BE621C3A3960A54E710\
             C375F26375D7014103A4B54330C198AF126116D2276E11715F693877FAD7EF09CADB094AE91E1A1597",
        ).unwrap();
        let prime_q = BigNum::from_hex_str(
            "3FB32C9B73134D0B2E77506660EDBD484CA7B18F21EF205407F4793A1A0BA12510DBC15077BE463FFF4FED\
             4AAC0BB555BE3A6C1B0C6B47B1BC3773BF7E8C6F62901228F8C28CBB18A55AE31341000A650196F931C77A\
             57F2DDF463E5E9EC144B777DE62AAAB8A8628AC376D282D6ED3864E67982428EBC831D14348F6F2F9193B5\
             045AF2767164E1DFC967C1FB3F2E55A4BD1BFFE83B9C80D052B985D182EA0ADB2A3B7313D3FE14C8484B1E\
             052588B9B7D2BBD2DF016199ECD06E1557CD0915B3353BBB64E0EC377FD028370DF92B52C7891428CDC67E\
             B6184B523D1DB246C32F63078490F00EF8D647D148D47954515E2327CFEF98C582664B4C0F6CC41659",
        ).unwrap();
        let generator = BigNum::from_hex_str(
            "8CF83642A709A097B447997640129DA299B1A47D1EB3750BA308B0FE64F5FBD3",
        )
        .unwrap();
        let dh = Dh::from_params(
            prime_p.to_owned().unwrap(),
            generator.to_owned().unwrap(),
            prime_q.to_owned().unwrap(),
        )
        .unwrap();
        ctx.set_tmp_dh(&dh).unwrap();

        assert_eq!(dh.prime_p(), &prime_p);
        assert_eq!(dh.prime_q().unwrap(), &prime_q);
        assert_eq!(dh.generator(), &generator);
    }

    #[test]
    fn test_dh_from_pem() {
        let mut ctx = SslContext::builder(SslMethod::tls()).unwrap();
        let params = include_bytes!("../test/dhparams.pem");
        let dh = Dh::params_from_pem(params).unwrap();
        ctx.set_tmp_dh(&dh).unwrap();
    }

    #[test]
    fn test_dh_from_der() {
        let params = include_bytes!("../test/dhparams.pem");
        let dh = Dh::params_from_pem(params).unwrap();
        let der = dh.params_to_der().unwrap();
        Dh::params_from_der(&der).unwrap();
    }

    #[test]
    #[cfg(ossl102)]
    fn test_dh_generate_key_compute_key() {
        let dh1 = Dh::get_2048_224().unwrap().generate_key().unwrap();
        let dh2 = Dh::get_2048_224().unwrap().generate_key().unwrap();

        let shared_a = dh1.compute_key(dh2.public_key()).unwrap();
        let shared_b = dh2.compute_key(dh1.public_key()).unwrap();

        assert_eq!(shared_a, shared_b);
    }

    #[test]
    fn test_dh_generate_params_generate_key_compute_key() {
        let dh_params1 = Dh::generate_params(512, 2).unwrap();
        let dh_params2 = Dh::from_pqg(
            dh_params1.prime_p().to_owned().unwrap(),
            None,
            dh_params1.generator().to_owned().unwrap(),
        )
        .unwrap();

        let dh1 = dh_params1.generate_key().unwrap();
        let dh2 = dh_params2.generate_key().unwrap();

        let shared_a = dh1.compute_key(dh2.public_key()).unwrap();
        let shared_b = dh2.compute_key(dh1.public_key()).unwrap();

        assert_eq!(shared_a, shared_b);
    }
}