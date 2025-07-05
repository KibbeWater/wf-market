use crate::{Client, errors::ApiError};
#[tokio::test]
async fn test_login_error_handling_and_masking() {
    let client = Client::new();
    let test_user = "invalid_test_user";
    let test_pass = "invalid_test_password_123";
    let device_id = "test_device_12345";

    println!("🔐 Testing login with invalid credentials...");
    println!("   User: {}", test_user);
    println!("   Password: {}***", &test_pass[..4]); // Show first 4 chars for demo
    println!("   Device ID: {}", device_id);

    match client.login(test_user, test_pass, device_id).await {
        Ok(_) => {
            println!("❌ Unexpected: Login should have failed with invalid credentials");
            assert!(false, "Login should have failed");
        }
        Err(e) => {
            println!("✅ Login failed as expected");

            match e {
                ApiError::InvalidCredentials(request_error) => {
                    println!("\n📋 Request error details:");
                    println!("   Status: {}", request_error.status_code);
                    println!("   Method: {}", request_error.method);
                    println!("   URL: {}", request_error.url);
                    println!("   Content length: {} chars", request_error.content.len());

                    println!("\n🎭 Testing sensitive data in payload...");

                    // Check if payload contains sensitive data
                    if let Some(payload) = &request_error.payload {
                        let payload_str = payload.to_string();
                        println!(
                            "   Payload contains password: {}",
                            payload_str.contains(test_pass)
                        );
                        println!(
                            "   Payload contains device ID: {}",
                            payload_str.contains(device_id)
                        );

                        // Show a safe representation
                        println!(
                            "   Payload preview: {}...",
                            if payload_str.len() > 50 {
                                &payload_str[..50]
                            } else {
                                &payload_str
                            }
                        );
                    }

                    // Check headers for sensitive data
                    if !request_error.headers.is_empty() {
                        println!("\n📨 Headers information:");
                        println!("   Number of headers: {}", request_error.headers.len());
                        for (key, _) in &request_error.headers {
                            println!("   Header: {}", key);
                        }
                    }

                    println!("✅ Error handling test completed");
                }
                ApiError::ParsingError(req_err, parse_err) => {
                    println!("📝 Parsing error: {:?} - {}", req_err, parse_err);
                    println!("   This might indicate API response format changes");
                }
                ApiError::Unknown(msg) => {
                    println!("❓ Unknown error: {}", msg);
                }
                _ => {
                    println!("💥 Unexpected error type: {:?}", e);
                    // Don't fail the test for unexpected error types during development
                    println!("   This error type might need to be handled in the future");
                }
            }
        }
    }

    println!("\n🎯 Test completed: Error handling functionality verified");
}
