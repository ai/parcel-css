use crate::compat::Feature;
use crate::declaration::DeclarationBlock;
use crate::properties::custom::UnparsedProperty;
use crate::properties::Property;
use crate::rules::supports::{SupportsCondition, SupportsRule};
use crate::rules::{style::StyleRule, CssRule, CssRuleList};
use crate::selector::{Direction, PseudoClass};
use crate::targets::Browsers;
use crate::vendor_prefix::VendorPrefix;
use parcel_selectors::parser::Component;

#[derive(Debug)]
pub(crate) struct SupportsEntry<'i> {
  pub condition: SupportsCondition<'i>,
  pub declarations: Vec<Property<'i>>,
  pub important_declarations: Vec<Property<'i>>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum DeclarationContext {
  None,
  StyleRule,
  Keyframes,
  StyleAttribute,
}

#[derive(Debug)]
pub(crate) struct PropertyHandlerContext<'i> {
  pub targets: Option<Browsers>,
  pub is_important: bool,
  supports: Vec<SupportsEntry<'i>>,
  ltr: Vec<Property<'i>>,
  rtl: Vec<Property<'i>>,
  pub context: DeclarationContext,
}

impl<'i> PropertyHandlerContext<'i> {
  pub fn new(targets: Option<Browsers>) -> Self {
    PropertyHandlerContext {
      targets,
      is_important: false,
      supports: Vec::new(),
      ltr: Vec::new(),
      rtl: Vec::new(),
      context: DeclarationContext::None,
    }
  }

  pub fn is_supported(&self, feature: Feature) -> bool {
    // Don't convert logical properties in style attributes because
    // our fallbacks rely on extra rules to define --ltr and --rtl.
    if self.context == DeclarationContext::StyleAttribute {
      return true;
    }

    if let Some(targets) = self.targets {
      feature.is_compatible(targets)
    } else {
      true
    }
  }

  pub fn add_logical_rule(&mut self, ltr: Property<'i>, rtl: Property<'i>) {
    self.ltr.push(ltr);
    self.rtl.push(rtl);
  }

  pub fn get_logical_rules(&mut self, style_rule: &StyleRule<'i>) -> Vec<CssRule<'i>> {
    // TODO: :dir/:lang raises the specificity of the selector. Use :where to lower it?
    let mut dest = Vec::new();

    macro_rules! rule {
      ($dir: ident, $decls: ident) => {
        let mut selectors = style_rule.selectors.clone();
        for selector in &mut selectors.0 {
          selector.append(Component::NonTSPseudoClass(PseudoClass::Dir(Direction::$dir)));
        }

        let rule = StyleRule {
          selectors,
          vendor_prefix: VendorPrefix::None,
          declarations: DeclarationBlock {
            declarations: std::mem::take(&mut self.$decls),
            important_declarations: vec![],
          },
          rules: CssRuleList(vec![]),
          loc: style_rule.loc.clone(),
        };

        dest.push(CssRule::Style(rule));
      };
    }

    if !self.ltr.is_empty() {
      rule!(Ltr, ltr);
    }

    if !self.rtl.is_empty() {
      rule!(Rtl, rtl);
    }

    dest
  }

  pub fn add_conditional_property(&mut self, condition: SupportsCondition<'i>, property: Property<'i>) {
    if self.context != DeclarationContext::StyleRule {
      return;
    }

    if let Some(entry) = self.supports.iter_mut().find(|supports| condition == supports.condition) {
      if self.is_important {
        entry.important_declarations.push(property);
      } else {
        entry.declarations.push(property);
      }
    } else {
      let mut important_declarations = Vec::new();
      let mut declarations = Vec::new();
      if self.is_important {
        important_declarations.push(property);
      } else {
        declarations.push(property);
      }
      self.supports.push(SupportsEntry {
        condition,
        important_declarations,
        declarations,
      });
    }
  }

  pub fn add_unparsed_fallbacks(&mut self, unparsed: &mut UnparsedProperty<'i>) {
    if self.context != DeclarationContext::StyleRule && self.context != DeclarationContext::StyleAttribute {
      return;
    }

    if let Some(targets) = self.targets {
      let fallbacks = unparsed.value.get_fallbacks(targets);
      for (condition, fallback) in fallbacks {
        self.add_conditional_property(
          condition,
          Property::Unparsed(UnparsedProperty {
            property_id: unparsed.property_id.clone(),
            value: fallback,
          }),
        );
      }
    }
  }

  pub fn get_supports_rules(&mut self, style_rule: &StyleRule<'i>) -> Vec<CssRule<'i>> {
    if self.supports.is_empty() {
      return Vec::new();
    }

    let mut dest = Vec::new();
    let supports = std::mem::take(&mut self.supports);
    for entry in supports {
      dest.push(CssRule::Supports(SupportsRule {
        condition: entry.condition,
        rules: CssRuleList(vec![CssRule::Style(StyleRule {
          selectors: style_rule.selectors.clone(),
          vendor_prefix: VendorPrefix::None,
          declarations: DeclarationBlock {
            declarations: entry.declarations,
            important_declarations: entry.important_declarations,
          },
          rules: CssRuleList(vec![]),
          loc: style_rule.loc.clone(),
        })]),
        loc: style_rule.loc.clone(),
      }));
    }

    dest
  }
}
