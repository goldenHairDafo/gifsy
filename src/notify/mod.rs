use notify_rust::Notification;
use std::result;

pub type Result = result::Result<(), String>;

#[cfg(target_os = "linux")]
pub fn notify(sum: &str, msg: &str) -> Result {
    let res = Notification::new().summary(sum).body(msg).show();
    match res
    {
        Ok(_) => Ok(()),
        Err(_) => Err("can't notify".to_owned()),
    }
}
#[cfg(target_os = "macos")]
pub fn notify(sum: &str, msg: &str) -> Result {
    let res = Notification::new().summary(sum).body(msg).show();
    Ok(())
}
#[cfg(other)]
pub fn notify(sum: &str, msg: &str) -> Result {
    println!("{}", sum);
    println!("{}", msg);
    Ok(())
}
