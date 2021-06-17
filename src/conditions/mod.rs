use crate::config::component::ComponentDescription;
use crate::event::Event;
use serde::{Deserialize, Serialize};

pub mod check_fields;
pub mod is_log;
pub mod is_metric;
pub mod remap;

pub use check_fields::CheckFieldsConfig;

pub use self::remap::RemapConfig;

pub trait Condition: Send + Sync + dyn_clone::DynClone {
    fn check(&self, e: &Event) -> bool;

    /// Provides context for a failure. This is potentially mildly expensive if
    /// it involves string building and so should be avoided in hot paths.
    fn check_with_context(&self, e: &Event) -> Result<(), String> {
        if self.check(e) {
            Ok(())
        } else {
            Err("condition failed".into())
        }
    }
}

dyn_clone::clone_trait_object!(Condition);

#[typetag::serde(tag = "type")]
pub trait ConditionConfig: std::fmt::Debug + Send + Sync + dyn_clone::DynClone {
    fn build(&self) -> crate::Result<Box<dyn Condition>>;
}

dyn_clone::clone_trait_object!(ConditionConfig);

pub type ConditionDescription = ComponentDescription<Box<dyn ConditionConfig>>;

inventory::collect!(ConditionDescription);

/// A condition can either be a raw string such as
/// `condition = '.message == "hooray"'`.
/// In this case it is turned into a Remap condition.
/// Otherwise it is a condition such as:
///
/// condition.type = 'check_fields'
/// condition."message.equals" = 'hooray'
///
///
/// It is important to note that because the way this is
/// structured, it is wrong to flatten a field that contains
/// an AnyCondition:
///
/// #[serde(flatten)]
/// condition: AnyCondition,
///
/// This will result in an error when serializing to json
/// which we need to do when determining which transforms have changed
/// when a config is reloaded.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum AnyCondition {
    String(String),
    Map(Box<dyn ConditionConfig>),
}

impl AnyCondition {
    pub fn build(&self) -> crate::Result<Box<dyn Condition>> {
        match self {
            AnyCondition::String(s) => RemapConfig { source: s.clone() }.build(),
            AnyCondition::Map(m) => m.build(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[derive(Deserialize, Debug)]
    struct Test {
        condition: AnyCondition,
    }

    #[test]
    fn deserialize_anycondition_default() {
        let conf: Test = toml::from_str(r#"condition = ".nork == false""#).unwrap();
        assert_eq!(
            r#"String(".nork == false")"#,
            format!("{:?}", conf.condition)
        )
    }

    #[test]
    fn deserialize_anycondition_check_fields() {
        let conf: Test = toml::from_str(indoc! {r#"
            condition.type = "check_fields"
            condition."norg.equals" = "nork"
        "#})
        .unwrap();

        assert_eq!(
            r#"Map(CheckFieldsConfig { predicates: {"norg.equals": "nork"} })"#,
            format!("{:?}", conf.condition)
        )
    }

    #[test]
    fn deserialize_anycondition_remap() {
        let conf: Test = toml::from_str(indoc! {r#"
            condition.type = "remap"
            condition.source = '.nork == true'
        "#})
        .unwrap();

        assert_eq!(
            r#"Map(RemapConfig { source: ".nork == true" })"#,
            format!("{:?}", conf.condition)
        )
    }
}
