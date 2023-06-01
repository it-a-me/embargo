///trait that allows a struct to function as a slint widget
pub trait Widget {
    ///update internal state.  It is recomended to maintain an internal timer that limits the frequency of updates
    fn update(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    ///the value to be displaye by the widget
    fn value_str(&self) -> &str;
}
