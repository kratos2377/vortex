use axum::body::Body;
use axum::extract::Request;
use crate::{ctx::Ctx, errors::Error};
use crate::errors::Result;

use axum::middleware::Next;
use axum::response::Response;
use tower_cookies::{Cookie, Cookies};
use lazy_regex::regex_captures;

const AUTH_TOKEN: &str = "auth-token";


pub async fn middleware_require_auth(
    ctx: Result<Ctx>,
    req: Request<Body>,
    next: Next,
) -> Result<Response> {

	ctx?;

	Ok(next.run(req).await)

}


pub async fn middleware_context_resolver(
    cookies: Cookies,
	mut req: Request<Body>,
	next: Next,
) -> Result<Response> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");

	let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

	// Compute Result<Ctx>.
	let result_ctx = match auth_token
		.ok_or(Error::AuthFailNoAuthTokenCookie)
		.and_then(parse_token)
	{
		Ok((user_id, _exp, _sign)) => {
			// TODO: Token components validations.
			Ok(Ctx::new(user_id))
		}
		Err(e) => Err(e),
	};

	if result_ctx.is_err()
		&& !matches!(result_ctx, Err(Error::AuthFailNoAuthTokenCookie))
	{
		cookies.remove(Cookie::from(AUTH_TOKEN))
	}

	// Store the ctx_result in the request extension.
	req.extensions_mut().insert(result_ctx);

	Ok(next.run(req).await)
}

pub fn parse_token(token: String) -> Result<(u64, String , String)> {
    let (_whole, user_id, exp, sign) = regex_captures!(
		r#"^user-(\d+)\.(.+)\.(.+)"#, // a literal regex
		&token
	)
	.ok_or(Error::AuthFailTokenWrongFormat)?;

	let user_id: u64 = user_id
		.parse()
		.map_err(|_| Error::AuthFailTokenWrongFormat)?;

	Ok((user_id, exp.to_string(), sign.to_string()))

}