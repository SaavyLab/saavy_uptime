# hb-auth

**Identity and permissions for Cloudflare Workers.**

`hb-auth` provides drop-in Cloudflare Access JWT validation with a strongly-typed permission DSL. It handles key rotation, signature verification, and identity extraction so you can focus on your business logic.

Part of the [**hb** stack](../hb-README.md).

---

## Features

- ✅ **Zero-Config Validation** – Automatically fetches and caches JWKS from your Cloudflare Access team domain.
- ✅ **Type-Safe Identity** – Extract a strongly-typed `User` struct from requests.
- ✅ **Role-Based Access** – Map Access Groups (e.g., "super-admins") to internal Rust enums automatically.
- ✅ **Framework Agnostic** – First-class support for `axum` (via extractors) and raw `worker::Request`.

---

## Installation

```toml
[dependencies]
hb-auth = { path = "../hb-auth", features = ["axum"] } # Enable axum support
```

---

## Quick Start (Axum)

### 1. Configure

In your router setup, initialize the config and add it to your state. Your state must implement `HasAuthConfig`.

```rust
use hb_auth::{AuthConfig, HasAuthConfig};

#[derive(Clone)]
struct AppState {
    auth_config: AuthConfig,
    // ... other state
}

impl HasAuthConfig for AppState {
    fn auth_config(&self) -> &AuthConfig {
        &self.auth_config
    }
}

fn router(env: Env) -> Router {
    let auth_config = AuthConfig::new(
        "https://my-team.cloudflareaccess.com",
        env.var("ACCESS_AUD").unwrap().to_string()
    );
    
    let state = AppState { auth_config };
    
    Router::new()
        .route("/secure", get(handler))
        .with_state(state)
}
```

### 2. Protect Routes

Add `auth: User` to your handler arguments. The handler will only run if a valid Cloudflare Access JWT is present.

```rust
use hb_auth::User;

async fn handler(auth: User) -> &'static str {
    format!("Hello, {}!", auth.email())
}
```

---

## Advanced: Role-Based Access

You can map Cloudflare Access Groups (available in the JWT `groups` claim) to your own internal roles.

### 1. Define Roles

```rust
use hb_auth::{RoleMapper, Claims};

#[derive(Debug, PartialEq, Clone)]
pub enum Role {
    Admin,
    Editor,
    Viewer,
}

impl RoleMapper for Role {
    fn from_claims(claims: &Claims) -> Vec<Self> {
        let mut roles = vec![];
        
        // Map groups from Cloudflare Access
        for group in &claims.groups {
            match group.as_str() {
                "000-my-app-admins" => roles.push(Role::Admin),
                "000-my-app-editors" => roles.push(Role::Editor),
                _ => {}
            }
        }
        
        // Or map specific emails
        if claims.email.ends_with("@saavylab.com") {
            roles.push(Role::Viewer);
        }
        
        roles
    }
}
```

### 2. Enforce Permissions

Use `User<Role>` instead of the default `User`.

```rust
async fn delete_db(auth: User<Role>) -> Result<impl IntoResponse, StatusCode> {
    if !auth.has_role(Role::Admin) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // ... unsafe logic
    Ok("Deleted")
}
```

---

## Usage with Raw Workers

If you aren't using Axum, you can still use `hb-auth` to validate requests.

```rust
use hb_auth::User;

async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let config = AuthConfig::new(/* ... */);
    
    let user = match User::<()>::from_worker_request(&req, &config).await {
        Ok(u) => u,
        Err(e) => return Response::error(e, 401),
    };
    
    Response::ok(format!("Welcome {}", user.email()))
}
```

---

## Configuring Cloudflare Access

To get the `groups` claim in your JWT:

1. Go to **Zero Trust Dashboard** > **Access** > **Applications**.
2. Edit your application.
3. Under **Settings** (or "Overview" -> "Edit"), find **OIDC Claims** or **Additional Settings**.
4. Enable **Groups** (this might require adding "groups" to the scope depending on your configuration).
5. Ensure the groups you want to map are assigned to the application policy.

The `audience` (AUD) is found in the **Overview** tab of your Access Application.

---

## License

MIT
