use regex::Regex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CipherError {
    #[error("JS execution error: {0}")]
    JsError(String),
    #[error("Failed to parse cipher URL")]
    ParseError,
    #[error("Signature extraction failed")]
    SignatureNotFound,
}

/// Dynamic solver for YouTube player signature and n-sig challenges.
/// Pure Rust and thread-safe (Send + Sync).
#[derive(Default, Clone)]
pub struct CipherSolver;

impl CipherSolver {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates dynamic decipher operations for signature cipher query string.
    pub async fn solve_signature_cipher(
        &self,
        cipher_query: &str,
        player_js: Option<&str>,
    ) -> Result<String, CipherError> {
        let parsed = url::form_urlencoded::parse(cipher_query.as_bytes());
        let mut target_url = None;
        let mut signature = None;
        let mut sig_param = "sig".to_string();

        for (k, v) in parsed {
            match k.as_ref() {
                "url" => target_url = Some(v.to_string()),
                "s" => signature = Some(v.to_string()),
                "sp" => sig_param = v.to_string(),
                _ => {}
            }
        }

        let base_url = target_url.ok_or(CipherError::ParseError)?;
        let s = signature.ok_or(CipherError::SignatureNotFound)?;

        let solved_sig = if let Some(js_code) = player_js {
            self.eval_js_signature(&s, js_code).unwrap_or(s)
        } else {
            s
        };

        let mut url = url::Url::parse(&base_url).map_err(|_| CipherError::ParseError)?;
        url.query_pairs_mut().append_pair(&sig_param, &solved_sig);

        Ok(url.to_string())
    }

    /// Evaluates n-parameter challenge transformation to avoid 403 bandwidth throttling.
    pub async fn solve_n_parameter(&self, url: &str, player_js: Option<&str>) -> Result<String, CipherError> {
        let mut parsed_url = url::Url::parse(url).map_err(|_| CipherError::ParseError)?;
        let n_val = parsed_url
            .query_pairs()
            .find(|(k, _)| k == "n")
            .map(|(_, v)| v.to_string());

        if let (Some(n), Some(js_code)) = (n_val, player_js) {
            if let Ok(transformed_n) = self.eval_n_transform(&n, js_code) {
                let mut pairs: Vec<(String, String)> = parsed_url
                    .query_pairs()
                    .filter(|(k, _)| k != "n")
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                pairs.push(("n".to_string(), transformed_n));

                parsed_url.set_query(None);
                for (k, v) in pairs {
                    parsed_url.query_pairs_mut().append_pair(&k, &v);
                }
            }
        }

        Ok(parsed_url.to_string())
    }

    fn eval_js_signature(&self, sig: &str, js_code: &str) -> Result<String, CipherError> {
        let mut ctx = boa_engine::Context::default();

        let re_entry = Regex::new(r#"(?:\b[cs]\s*&&\s*[adf]\.set\([^,]+\s*,\s*encodeURIComponent\s*\(\s*([a-zA-Z0-9$]+)\("#).unwrap();
        if let Some(caps) = re_entry.captures(js_code) {
            let fn_name = &caps[1];
            let script = format!("{}\n{}(\"{}\");", js_code, fn_name, sig);
            let val = ctx
                .eval(boa_engine::Source::from_bytes(&script))
                .map_err(|e| CipherError::JsError(e.to_string()))?;
            return Ok(val.to_string(&mut ctx).map_err(|e| CipherError::JsError(e.to_string()))?.to_std_string_escaped());
        }

        Err(CipherError::SignatureNotFound)
    }

    fn eval_n_transform(&self, n: &str, js_code: &str) -> Result<String, CipherError> {
        let mut ctx = boa_engine::Context::default();

        let re_n = Regex::new(r#"([a-zA-Z0-9$]+)\s*=\s*function\([a-zA-Z0-9$]+\)\s*\{var\s+[a-zA-Z0-9$]+\s*=\s*[a-zA-Z0-9$]+\.split\(""#).unwrap();
        if let Some(caps) = re_n.captures(js_code) {
            let fn_name = &caps[1];
            let script = format!("{}\n{}(\"{}\");", js_code, fn_name, n);
            let val = ctx
                .eval(boa_engine::Source::from_bytes(&script))
                .map_err(|e| CipherError::JsError(e.to_string()))?;
            return Ok(val.to_string(&mut ctx).map_err(|e| CipherError::JsError(e.to_string()))?.to_std_string_escaped());
        }

        Err(CipherError::JsError("n-transform function not found".to_string()))
    }
}
