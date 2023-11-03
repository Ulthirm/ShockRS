use rosc::OscType;

pub async fn handler(touchpoint: String,touchpoint_args: Vec<OscType>) {
    log::debug!("Openshock Touchpoint Handler");
    log::warn!("unimplemented API");
    for arg in touchpoint_args {
        log::debug!("Touchpoint Argument: {:?}", arg);
    }
}