use crate::commons::BindingDestinationType;
use crate::requests::XArguments;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingDeletionParams<'a> {
    pub virtual_host: &'a str,
    pub source: &'a str,
    pub destination: &'a str,
    pub destination_type: BindingDestinationType,
    pub routing_key: &'a str,
    pub arguments: XArguments,
}
