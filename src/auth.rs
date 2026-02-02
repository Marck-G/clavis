use crate::ldap;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    success: bool,
    error: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
    confirm_password: String,
}

#[derive(Serialize)]
pub struct ChangePasswordResponse {
    success: bool,
    message: String,
}

#[derive(Debug)]
pub enum PasswordValidationError {
    TooShort,
    NoUppercase,
    NoLowercase,
    NoDigit,
    NoSpecialChar,
    Mismatch,
}

impl std::fmt::Display for PasswordValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordValidationError::TooShort => {
                write!(f, "La contraseña debe tener al menos 8 caracteres")
            }
            PasswordValidationError::NoUppercase => {
                write!(f, "La contraseña debe contener al menos una mayúscula")
            }
            PasswordValidationError::NoLowercase => {
                write!(f, "La contraseña debe contener al menos una minúscula")
            }
            PasswordValidationError::NoDigit => {
                write!(f, "La contraseña debe contener al menos un número")
            }
            PasswordValidationError::NoSpecialChar => write!(
                f,
                "La contraseña debe contener al menos un carácter especial"
            ),
            PasswordValidationError::Mismatch => write!(f, "Las contraseñas no coinciden"),
        }
    }
}

pub async fn login(session: Session, Json(payload): Json<LoginRequest>) -> Response {
    match ldap::authenticate_user(&payload.username, &payload.password).await {
        Ok(_) => {
            // Authentication successful, create session
            if let Err(e) = session.insert("user", payload.username).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        success: false,
                        error: format!("Failed to create session: {}", e),
                    }),
                )
                    .into_response();
            }

            (
                StatusCode::OK,
                Json(LoginResponse {
                    success: true,
                    message: "Authentication successful".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                success: false,
                error: e,
            }),
        )
            .into_response(),
    }
}

pub async fn logout(session: Session) -> Response {
    session.clear().await;
    (
        StatusCode::OK,
        Json(LoginResponse {
            success: true,
            message: "Logged out successfully".to_string(),
        }),
    )
        .into_response()
}

fn validate_password(password: &str) -> Result<(), PasswordValidationError> {
    if password.len() < 8 {
        return Err(PasswordValidationError::TooShort);
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(PasswordValidationError::NoUppercase);
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(PasswordValidationError::NoLowercase);
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err(PasswordValidationError::NoDigit);
    }

    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(PasswordValidationError::NoSpecialChar);
    }

    Ok(())
}

pub async fn change_password(
    session: Session,
    Json(payload): Json<ChangePasswordRequest>,
) -> Response {
    // Check if user is authenticated
    let username: Option<String> = session.get("user").await.unwrap_or(None);
    if username.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                success: false,
                error: "No active session".to_string(),
            }),
        )
            .into_response();
    }
    let username = username.unwrap();

    // Validate passwords match
    if payload.new_password != payload.confirm_password {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: PasswordValidationError::Mismatch.to_string(),
            }),
        )
            .into_response();
    }

    // Validate password policy
    match validate_password(&payload.new_password) {
        Ok(_) => (),
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    success: false,
                    error: e.to_string(),
                }),
            )
                .into_response();
        }
    }

    // Authenticate user with current password before changing it
    match ldap::authenticate_user(&username, &payload.current_password).await {
        Ok(_) => {
            // User authenticated successfully, proceed with password change
        }
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    success: false,
                    error: format!("Authentication failed: {}", e),
                }),
            )
                .into_response();
        }
    }

    // Change password in LDAP
    match ldap::change_user_password(&username, &payload.new_password).await {
        Ok(_) => {
            // Send email notification
            if let Err(e) = send_password_change_notification(&username).await {
                tracing::warn!("Failed to send notification email: {}", e);
            }

            (
                StatusCode::OK,
                Json(ChangePasswordResponse {
                    success: true,
                    message: "Contraseña actualizada correctamente".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                success: false,
                error: format!("Failed to change password: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn send_password_change_notification(username: &str) -> Result<(), String> {
    // For now, just log the notification attempt
    // In a real implementation, this would send an email
    tracing::info!("Password change notification sent to user: {}", username);
    Ok(())
}
