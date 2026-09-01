// Android WebView implementation using JNI
// Follows the pattern from uad-shizuku/android_activity.rs

use jni::objects::{GlobalRef, JObject, JValue};
use jni::JavaVM;
use std::error::Error;
use std::sync::Arc;

/// Android WebView wrapper using JNI to call android.webkit.WebView
pub struct AndroidWebView {
    vm: JavaVM,
    webview: GlobalRef,
}

impl AndroidWebView {
    /// Create a new Android WebView
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let ctx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(ctx.vm() as _) }?;
        let activity = unsafe { JObject::from_raw(ctx.context() as _) };
        let mut env = vm.attach_current_thread()?;

        // Find android.webkit.WebView class
        let webview_class = env.find_class("android/webkit/WebView")?;

        // Create WebView instance: new WebView(Context context)
        let webview_local = env.new_object(
            &webview_class,
            "(Landroid/content/Context;)V",
            &[JValue::Object(&activity)],
        )?;

        // Get WebSettings and enable JavaScript
        let settings = env
            .call_method(
                &webview_local,
                "getSettings",
                "()Landroid/webkit/WebSettings;",
                &[],
            )?
            .l()?;

        env.call_method(
            &settings,
            "setJavaScriptEnabled",
            "(Z)V",
            &[JValue::Bool(1)],
        )?;

        // Enable DOM storage
        env.call_method(
            &settings,
            "setDomStorageEnabled",
            "(Z)V",
            &[JValue::Bool(1)],
        )?;

        // Load initial URL
        let url_string = env.new_string(url)?;
        env.call_method(
            &webview_local,
            "loadUrl",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&url_string)],
        )?;

        // Create global reference to keep WebView alive
        let webview = env.new_global_ref(&webview_local)?;

        Ok(Self { vm, webview })
    }

    /// Load a URL
    pub fn load_url(&self, url: &str) -> Result<(), Box<dyn Error>> {
        let mut env = self.vm.attach_current_thread()?;
        let url_string = env.new_string(url)?;

        env.call_method(
            self.webview.as_obj(),
            "loadUrl",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&url_string)],
        )?;

        Ok(())
    }

    /// Navigate back
    pub fn go_back(&self) -> Result<(), Box<dyn Error>> {
        let mut env = self.vm.attach_current_thread()?;

        env.call_method(
            self.webview.as_obj(),
            "goBack",
            "()V",
            &[],
        )?;

        Ok(())
    }

    /// Navigate forward
    pub fn go_forward(&self) -> Result<(), Box<dyn Error>> {
        let mut env = self.vm.attach_current_thread()?;

        env.call_method(
            self.webview.as_obj(),
            "goForward",
            "()V",
            &[],
        )?;

        Ok(())
    }

    /// Check if can go back
    pub fn can_go_back(&self) -> Result<bool, Box<dyn Error>> {
        let mut env = self.vm.attach_current_thread()?;

        let result = env.call_method(
            self.webview.as_obj(),
            "canGoBack",
            "()Z",
            &[],
        )?;

        Ok(result.z()?)
    }

    /// Check if can go forward
    pub fn can_go_forward(&self) -> Result<bool, Box<dyn Error>> {
        let mut env = self.vm.attach_current_thread()?;

        let result = env.call_method(
            self.webview.as_obj(),
            "canGoForward",
            "()Z",
            &[],
        )?;

        Ok(result.z()?)
    }

    /// Get current URL
    pub fn get_url(&self) -> Result<String, Box<dyn Error>> {
        let mut env = self.vm.attach_current_thread()?;

        let result = env.call_method(
            self.webview.as_obj(),
            "getUrl",
            "()Ljava/lang/String;",
            &[],
        )?;

        let url_obj = result.l()?;
        if url_obj.is_null() {
            return Ok(String::new());
        }

        let url_jstring = env.get_string((&url_obj).into())?;
        Ok(url_jstring.into())
    }

    /// Reload current page
    pub fn reload(&self) -> Result<(), Box<dyn Error>> {
        let mut env = self.vm.attach_current_thread()?;

        env.call_method(
            self.webview.as_obj(),
            "reload",
            "()V",
            &[],
        )?;

        Ok(())
    }
}

impl Drop for AndroidWebView {
    fn drop(&mut self) {
        // Destroy WebView on drop
        if let Ok(mut env) = self.vm.attach_current_thread() {
            let _ = env.call_method(
                self.webview.as_obj(),
                "destroy",
                "()V",
                &[],
            );
        }
    }
}

// Android-specific stub methods for compatibility with desktop API
impl AndroidWebView {
    /// Set visibility (stub - WebView always visible on Android)
    pub fn set_visible(&self, _visible: bool) -> Result<(), Box<dyn Error>> {
        // Android WebView visibility is managed by the view hierarchy
        Ok(())
    }

    /// Focus the WebView
    pub fn focus(&self) -> Result<(), Box<dyn Error>> {
        let mut env = self.vm.attach_current_thread()?;

        env.call_method(
            self.webview.as_obj(),
            "requestFocus",
            "()Z",
            &[],
        )?;

        Ok(())
    }

    /// Set bounds (stub - Android manages layout differently)
    pub fn set_bounds(&self, _bounds: (i32, i32, u32, u32)) -> Result<(), Box<dyn Error>> {
        // Android view bounds are set by the layout manager
        // Would need to call setLayoutParams if we had the parent ViewGroup
        Ok(())
    }

    /// Evaluate JavaScript code
    pub fn evaluate_script(&self, script: &str) -> Result<(), Box<dyn Error>> {
        let mut env = self.vm.attach_current_thread()?;
        let script_string = env.new_string(script)?;

        env.call_method(
            self.webview.as_obj(),
            "evaluateJavascript",
            "(Ljava/lang/String;Landroid/webkit/ValueCallback;)V",
            &[JValue::Object(&script_string), JValue::Object(&JObject::null())],
        )?;

        Ok(())
    }
}
