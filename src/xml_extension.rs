use minidom::Element;

pub(crate) trait IgnoreNS {
    fn get_child_ignore_ns(&self, child_name: &str) -> Option<&Element>;
    fn get_children_ignore_ns(&self, child_name: &str) -> Vec<&Element>;
}

impl IgnoreNS for Element {
    fn get_child_ignore_ns(&self, child_name: &str) -> Option<&Element> {
        self.children().find(|&child| child.name().eq(child_name))
    }

    fn get_children_ignore_ns(&self, child_name: &str) -> Vec<&Element> {
        self.children()
            .filter(|&child| child.name().eq(child_name))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::xml_extension::IgnoreNS;
    use minidom::Element;

    const EMPTY_ELEMENT: Vec<&Element> = Vec::new();

    #[test]
    fn no_child() {
        let element = Element::bare("TestName", "TestNS");

        assert_eq!(element.get_child_ignore_ns("TestName"), None);
        assert_eq!(element.get_children_ignore_ns("TestName"), EMPTY_ELEMENT);
    }

    #[test]
    fn one_child() {
        let element = Element::builder("TestName", "TestNS")
            .append(Element::bare("TestA", "TestB"))
            .build();

        assert_eq!(
            element.get_child_ignore_ns("TestA"),
            Some(&Element::bare("TestA", "TestB"))
        );
        assert_eq!(
            element.get_children_ignore_ns("TestA"),
            vec![&Element::bare("TestA", "TestB")]
        );
    }

    #[test]
    fn multiple_children() {
        let element = Element::builder("TestName", "TestNS")
            .append(Element::bare("TestA", "TestB"))
            .append(Element::bare("TestA", "TestB"))
            .build();

        assert_eq!(
            element.get_child_ignore_ns("TestA"),
            Some(&Element::bare("TestA", "TestB"))
        );
        assert_eq!(
            element.get_children_ignore_ns("TestA"),
            vec![
                &Element::bare("TestA", "TestB"),
                &Element::bare("TestA", "TestB")
            ]
        );
    }
}
