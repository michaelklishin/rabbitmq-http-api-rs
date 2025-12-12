use crate::commons::BindingDestinationType;
use crate::requests::XArguments;
use crate::responses::BindingInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingDeletionParams<'a> {
    pub virtual_host: &'a str,
    pub source: &'a str,
    pub destination: &'a str,
    pub destination_type: BindingDestinationType,
    pub routing_key: &'a str,
    pub arguments: XArguments,
}

impl<'a> BindingDeletionParams<'a> {
    /// Creates deletion parameters from a [`BindingInfo`] response.
    ///
    /// This is useful when you have retrieved a binding from the API
    /// and want to delete it without manually extracting all fields.
    pub fn from_binding_info(binding: &'a BindingInfo) -> Self {
        let args = if binding.arguments.is_empty() {
            None
        } else {
            Some(binding.arguments.0.clone())
        };
        Self {
            virtual_host: &binding.vhost,
            source: &binding.source,
            destination: &binding.destination,
            destination_type: binding.destination_type.clone(),
            routing_key: &binding.routing_key,
            arguments: args,
        }
    }
}

impl<'a> From<&'a BindingInfo> for BindingDeletionParams<'a> {
    fn from(binding: &'a BindingInfo) -> Self {
        Self::from_binding_info(binding)
    }
}
