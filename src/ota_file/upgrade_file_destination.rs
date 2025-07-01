use ezsp::ember::Eui64;

#[derive(Debug)]
pub enum UpgradeFileDestination {
    Zigbee(Eui64),
    Thread([u8; 32]),
}
