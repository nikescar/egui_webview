// Android WebView implementation using separate AppCompatActivity
// Launches WebViewActivity to avoid "no available activity" crash with NativeActivity

use jni::objects::{JObject, JValue};
use jni::JavaVM;
use std::error::Error;

/// Android WebView wrapper that launches WebViewActivity
pub struct AndroidWebView {
    // Stub structure - actual WebView is in separate Activity
    // Future: Could store ActivityResult callback for bidirectional communication
}

impl AndroidWebView {
    /// Launch WebViewActivity with the given URL
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        launch_webview_activity(url)?;
        Ok(Self {})
    }

    /// Launch WebViewActivity with custom settings
    pub fn new_with_settings(url: &str, enable_js: bool, enable_dom: bool) -> Result<Self, Box<dyn Error>> {
        launch_webview_activity_with_settings(url, enable_js, enable_dom)?;
        Ok(Self {})
    }
}

/// Launch WebViewActivity with default settings
fn launch_webview_activity(url: &str) -> Result<(), Box<dyn Error>> {
    launch_webview_activity_with_settings(url, true, true)
}

/// Launch WebViewActivity with custom settings
fn launch_webview_activity_with_settings(
    url: &str,
    enable_js: bool,
    enable_dom: bool,
) -> Result<(), Box<dyn Error>> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm() as _) }?;
    let activity = unsafe { JObject::from_raw(ctx.context() as _) };
    let mut env = vm.attach_current_thread()?;

    // Find Intent and WebViewActivity classes
    let intent_class = env.find_class("android/content/Intent")?;
    let webview_class = env.find_class("app/dure/sijang/WebViewActivity")?;

    // Create Intent: new Intent(context, WebViewActivity.class)
    let intent = env.new_object(
        &intent_class,
        "(Landroid/content/Context;Ljava/lang/Class;)V",
        &[
            JValue::Object(&activity),
            JValue::Object(&webview_class.into()),
        ],
    )?;

    // Add URL as Intent extra
    let key_url = env.new_string("url")?;
    let value_url = env.new_string(url)?;
    env.call_method(
        &intent,
        "putExtra",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&key_url), JValue::Object(&value_url)],
    )?;

    // Add JavaScript enabled setting
    let key_js = env.new_string("enable_js")?;
    env.call_method(
        &intent,
        "putExtra",
        "(Ljava/lang/String;Z)Landroid/content/Intent;",
        &[JValue::Object(&key_js), JValue::Bool(enable_js as u8)],
    )?;

    // Add DOM storage setting
    let key_dom = env.new_string("enable_dom_storage")?;
    env.call_method(
        &intent,
        "putExtra",
        "(Ljava/lang/String;Z)Landroid/content/Intent;",
        &[JValue::Object(&key_dom), JValue::Bool(enable_dom as u8)],
    )?;

    // Launch Activity
    env.call_method(
        &activity,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)],
    )?;

    log::info!("Launched WebViewActivity with URL: {}", url);
    Ok(())
}

// Stub implementations for API compatibility
impl AndroidWebView {
    pub fn load_url(&self, _url: &str) -> Result<(), Box<dyn Error>> {
        log::warn!("load_url() not supported with separate Activity model");
        Ok(())
    }

    pub fn go_back(&self) -> Result<(), Box<dyn Error>> {
        log::warn!("go_back() not supported with separate Activity model");
        Ok(())
    }

    pub fn go_forward(&self) -> Result<(), Box<dyn Error>> {
        log::warn!("go_forward() not supported with separate Activity model");
        Ok(())
    }

    pub fn can_go_back(&self) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }

    pub fn can_go_forward(&self) -> Result<bool, Box<dyn Error>> {
        Ok(false)
    }

    pub fn get_url(&self) -> Result<String, Box<dyn Error>> {
        Ok(String::new())
    }

    pub fn reload(&self) -> Result<(), Box<dyn Error>> {
        log::warn!("reload() not supported with separate Activity model");
        Ok(())
    }
}
