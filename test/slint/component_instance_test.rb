# frozen_string_literal: true

require "test_helper"

module Slint
  class ComponentInstanceTest < Minitest::Test
    def test_definition
      compiler = Compiler.new
      compilation_result = compiler.build_from_source(source, "")
      component_definition = compilation_result.components.first
      component_instance = component_definition.create

      # Workaround: no equality operator for component definition, we compare by name
      assert_equal(component_definition.name, component_instance.definition.name)
    end

    def test_get_property
      compiler = Compiler.new
      compilation_result = compiler.build_from_source(source, "")
      component_definition = compilation_result.components.first
      component_instance = component_definition.create

      assert_equal(42, component_instance.get_property("int_property"))
      assert_equal("test-string-value", component_instance.get_property("text_prop"))
      assert(component_instance.get_property("bool_prop"))

      # TODO: until Image is implemented
      # assert_equal(some_image, component_instance.get_property("some_image"))
    end

    def test_get_property_raises_proper_error
      compiler = Compiler.new
      compilation_result = compiler.build_from_source(source, "")
      component_definition = compilation_result.components.first
      component_instance = component_definition.create

      assert_raises(Slint::Error) { component_instance.get_property("dupa") }
      assert_raises(Slint::Error) { component_instance.get_property("col_prop") }
    end

    private

    def source
      <<~SLINT
        export component App inherits Window {
          in-out property <int> int_property: 42;
          in-out property <string> text_prop: "test-string-value";
          in-out property <bool> bool_prop: true;
          in-out property <color> col_prop: #ffaaff;
          // to be implemented
          // in-out property <image> some_image: @image-url("")
        }
      SLINT
    end
  end
end
