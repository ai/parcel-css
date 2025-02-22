use crate::compat::Feature;
use crate::context::PropertyHandlerContext;
use crate::declaration::DeclarationList;
use crate::logical::PropertyCategory;
use crate::properties::{Property, PropertyId};
use crate::traits::PropertyHandler;
use crate::values::{length::LengthPercentageOrAuto, rect::Rect, size::Size2D};

macro_rules! side_handler {
  ($name: ident, $top: ident, $bottom: ident, $left: ident, $right: ident, $block_start: ident, $block_end: ident, $inline_start: ident, $inline_end: ident, $shorthand: ident, $block_shorthand: ident, $inline_shorthand: ident, $logical_shorthand: literal $(, $feature: ident)?) => {
    #[derive(Debug, Default)]
    pub(crate) struct $name<'i> {
      top: Option<LengthPercentageOrAuto>,
      bottom: Option<LengthPercentageOrAuto>,
      left: Option<LengthPercentageOrAuto>,
      right: Option<LengthPercentageOrAuto>,
      block_start: Option<Property<'i>>,
      block_end: Option<Property<'i>>,
      inline_start: Option<Property<'i>>,
      inline_end: Option<Property<'i>>,
      has_any: bool,
      category: PropertyCategory
    }

    impl<'i> PropertyHandler<'i> for $name<'i> {
      fn handle_property(&mut self, property: &Property<'i>, dest: &mut DeclarationList<'i>, context: &mut PropertyHandlerContext<'i>) -> bool {
        use Property::*;

        macro_rules! property {
          ($key: ident, $val: ident, $category: ident) => {{
            if PropertyCategory::$category != self.category {
              self.flush(dest, context);
            }
            self.$key = Some($val.clone());
            self.category = PropertyCategory::$category;
            self.has_any = true;
          }};
        }

        macro_rules! logical_property {
          ($prop: ident, $val: expr) => {{
            if self.category != PropertyCategory::Logical {
              self.flush(dest, context);
            }

            self.$prop = Some($val);
            self.category = PropertyCategory::Logical;
            self.has_any = true;
          }};
        }

        match &property {
          $top(val) => property!(top, val, Physical),
          $bottom(val) => property!(bottom, val, Physical),
          $left(val) => property!(left, val, Physical),
          $right(val) => property!(right, val, Physical),
          $block_start(_) => logical_property!(block_start, property.clone()),
          $block_end(_) => logical_property!(block_end, property.clone()),
          $inline_start(_) => logical_property!(inline_start, property.clone()),
          $inline_end(_) => logical_property!(inline_end, property.clone()),
          $block_shorthand(val) => {
            logical_property!(block_start, Property::$block_start(val.0.clone()));
            logical_property!(block_end, Property::$block_end(val.1.clone()));
          },
          $inline_shorthand(val) => {
            logical_property!(inline_start, Property::$inline_start(val.0.clone()));
            logical_property!(inline_end, Property::$inline_end(val.1.clone()));
          },
          $shorthand(val) => {
            // dest.clear();
            self.top = Some(val.0.clone());
            self.right = Some(val.1.clone());
            self.bottom = Some(val.2.clone());
            self.left = Some(val.3.clone());
            self.block_start = None;
            self.block_end = None;
            self.inline_start = None;
            self.inline_end = None;
            self.has_any = true;
          }
          Unparsed(val) if matches!(val.property_id, PropertyId::$top | PropertyId::$bottom | PropertyId::$left | PropertyId::$right | PropertyId::$block_start | PropertyId::$block_end | PropertyId::$inline_start | PropertyId::$inline_end | PropertyId::$block_shorthand | PropertyId::$inline_shorthand | PropertyId::$shorthand) => {
            // Even if we weren't able to parse the value (e.g. due to var() references),
            // we can still add vendor prefixes to the property itself.
            match &val.property_id {
              PropertyId::$block_start => logical_property!(block_start, property.clone()),
              PropertyId::$block_end => logical_property!(block_end, property.clone()),
              PropertyId::$inline_start => logical_property!(inline_start, property.clone()),
              PropertyId::$inline_end => logical_property!(inline_end, property.clone()),
              _ => {
                self.flush(dest, context);
                dest.push(property.clone());
              }
            }
          }
          _ => return false
        }

        true
      }

      fn finalize(&mut self, dest: &mut DeclarationList<'i>, context: &mut PropertyHandlerContext<'i>) {
        self.flush(dest, context);
      }
    }

    impl<'i> $name<'i> {
      fn flush(&mut self, dest: &mut DeclarationList<'i>, context: &mut PropertyHandlerContext<'i>) {
        use Property::*;

        if !self.has_any {
          return
        }

        self.has_any = false;

        let top = std::mem::take(&mut self.top);
        let bottom = std::mem::take(&mut self.bottom);
        let left = std::mem::take(&mut self.left);
        let right = std::mem::take(&mut self.right);
        let logical_supported = true $(&& context.is_supported(Feature::$feature))?;

        if (!$logical_shorthand || logical_supported) && top.is_some() && bottom.is_some() && left.is_some() && right.is_some() {
          let rect = Rect::new(top.unwrap(), right.unwrap(), bottom.unwrap(), left.unwrap());
          dest.push($shorthand(rect));
        } else {
          if let Some(val) = top {
            dest.push($top(val));
          }

          if let Some(val) = bottom {
            dest.push($bottom(val));
          }

          if let Some(val) = left {
            dest.push($left(val));
          }

          if let Some(val) = right {
            dest.push($right(val));
          }
        }

        let block_start = std::mem::take(&mut self.block_start);
        let block_end = std::mem::take(&mut self.block_end);
        let inline_start = std::mem::take(&mut self.inline_start);
        let inline_end = std::mem::take(&mut self.inline_end);

        macro_rules! logical_side {
          ($start: ident, $end: ident, $shorthand_prop: ident, $start_prop: ident, $end_prop: ident) => {
            if let (Some(Property::$start_prop(start)), Some(Property::$end_prop(end))) = (&$start, &$end) {
              let size = Size2D(start.clone(), end.clone());
              dest.push($shorthand_prop(size));
            } else {
              if let Some(val) = $start {
                dest.push(val);
              }

              if let Some(val) = $end {
                dest.push(val);
              }
            }
          };
        }

        macro_rules! prop {
          ($val: ident, $logical: ident, $physical: ident) => {
            match $val {
              Some(Property::$logical(val)) => {
                dest.push(Property::$physical(val));
              }
              Some(Property::Unparsed(val)) => {
                dest.push(Property::Unparsed(val.with_property_id(PropertyId::$physical)));
              }
              _ => {}
            }
          }
        }

        if logical_supported {
          logical_side!(block_start, block_end, $block_shorthand, $block_start, $block_end);
        } else {
          prop!(block_start, $block_start, $top);
          prop!(block_end, $block_end, $bottom);
        }

        if logical_supported {
          logical_side!(inline_start, inline_end, $inline_shorthand, $inline_start, $inline_end);
        } else if inline_start.is_some() || inline_end.is_some() {
          if matches!((&inline_start, &inline_end), (Some(Property::$inline_start(start)), Some(Property::$inline_end(end))) if start == end) {
            prop!(inline_start, $inline_start, $left);
            prop!(inline_end, $inline_end, $right);
          } else {
            macro_rules! logical_prop {
              ($val: ident, $logical: ident, $ltr: ident, $rtl: ident) => {
                match $val {
                  Some(Property::$logical(val)) => {
                    context.add_logical_rule(
                      Property::$ltr(val.clone()),
                      Property::$rtl(val)
                    );
                  }
                  Some(Property::Unparsed(val)) => {
                    context.add_logical_rule(
                      Property::Unparsed(val.with_property_id(PropertyId::$ltr)),
                      Property::Unparsed(val.with_property_id(PropertyId::$rtl))
                    );
                  }
                  _ => {}
                }
              }
            }

            logical_prop!(inline_start, $inline_start, $left, $right);
            logical_prop!(inline_end, $inline_end, $right, $left);
          }
        }
      }
    }
  };
}

side_handler!(
  MarginHandler,
  MarginTop,
  MarginBottom,
  MarginLeft,
  MarginRight,
  MarginBlockStart,
  MarginBlockEnd,
  MarginInlineStart,
  MarginInlineEnd,
  Margin,
  MarginBlock,
  MarginInline,
  false,
  LogicalMargin
);

side_handler!(
  PaddingHandler,
  PaddingTop,
  PaddingBottom,
  PaddingLeft,
  PaddingRight,
  PaddingBlockStart,
  PaddingBlockEnd,
  PaddingInlineStart,
  PaddingInlineEnd,
  Padding,
  PaddingBlock,
  PaddingInline,
  false,
  LogicalPadding
);

side_handler!(
  ScrollMarginHandler,
  ScrollMarginTop,
  ScrollMarginBottom,
  ScrollMarginLeft,
  ScrollMarginRight,
  ScrollMarginBlockStart,
  ScrollMarginBlockEnd,
  ScrollMarginInlineStart,
  ScrollMarginInlineEnd,
  ScrollMargin,
  ScrollMarginBlock,
  ScrollMarginInline,
  false
);

side_handler!(
  ScrollPaddingHandler,
  ScrollPaddingTop,
  ScrollPaddingBottom,
  ScrollPaddingLeft,
  ScrollPaddingRight,
  ScrollPaddingBlockStart,
  ScrollPaddingBlockEnd,
  ScrollPaddingInlineStart,
  ScrollPaddingInlineEnd,
  ScrollPadding,
  ScrollPaddingBlock,
  ScrollPaddingInline,
  false
);

side_handler!(
  InsetHandler,
  Top,
  Bottom,
  Left,
  Right,
  InsetBlockStart,
  InsetBlockEnd,
  InsetInlineStart,
  InsetInlineEnd,
  Inset,
  InsetBlock,
  InsetInline,
  true,
  LogicalInset
);
