# frozen_string_literal: true

require "test_helper"

module Slint
  class ComponentInstanceTest < Minitest::Test
    def setup
      compiler = Compiler.new
      compilation_result = compiler.build_from_source(source, "")
      @component_definition = compilation_result.components.first
      @component_instance = @component_definition.create
    end

    def test_definition
      # Workaround: no equality operator for component definition, we compare by name
      assert_equal(@component_definition.name, @component_instance.definition.name)
    end

    def test_property_accessor # rubocop:disable Metrics/AbcSize,Metrics/MethodLength
      assert_equal(42, @component_instance.get_property("int_property"))
      assert_in_delta(110.0, @component_instance.get_property("float_prop"))
      assert_equal("test-string-value", @component_instance.get_property("text_prop"))
      assert(@component_instance.get_property("bool_prop"))

      @component_instance.set_property("int_property", 10)
      @component_instance.set_property("float_prop", 10.5)
      @component_instance.set_property("text_prop", "new-string")
      @component_instance.set_property("bool_prop", false)

      assert_equal(10, @component_instance.get_property("int_property"))
      assert_in_delta(10.5, @component_instance.get_property("float_prop"))
      assert_equal("new-string", @component_instance.get_property("text_prop"))
      refute(@component_instance.get_property("bool_prop"))

      # TODO: until Image is implemented
      # assert_equal(some_image, component_instance.get_property("some_image"))
      assert_raises(Slint::Error) { @component_instance.get_property("non-existent") }
    end

    def test_global_property_accessor # rubocop:disable Metrics/AbcSize,Metrics/MethodLength
      assert_equal(100, @component_instance.get_global_property("Glob", "my_global_property"))
      assert_in_delta(30.3, @component_instance.get_global_property("Glob", "global_float_property"))
      assert_equal("global-string-value", @component_instance.get_global_property("Glob", "global_text_prop"))
      refute(@component_instance.get_global_property("Glob", "global_bool_prop"))

      @component_instance.set_global_property("Glob", "my_global_property", 200)
      @component_instance.set_global_property("Glob", "global_float_property", 20.3)
      @component_instance.set_global_property("Glob", "global_text_prop", "new-global-string")
      @component_instance.set_global_property("Glob", "global_bool_prop", true)

      assert_equal(200, @component_instance.get_global_property("Glob", "my_global_property"))
      assert_in_delta(20.3, @component_instance.get_global_property("Glob", "global_float_property"))
      assert_equal("new-global-string", @component_instance.get_global_property("Glob", "global_text_prop"))
      assert(@component_instance.get_global_property("Glob", "global_bool_prop"))

      assert_raises(Slint::Error) { @component_instance.get_global_property("Glob", "non-existent") }
      assert_raises(Slint::Error) { @component_instance.get_global_property("NonExistent", "my_global_property") }
      assert_raises(Slint::Error) { @component_instance.set_global_property("Glob", "non-existent", 10) }
      assert_raises(Slint::Error) { @component_instance.set_global_property("NonExistent", "my_global_property", 10) }
    end

    # def test_invoke_callback
    #   assert_equal("Hello World!", @component_instance.invoke("test-callback", "World!"))
    #   assert_raises(Slint::Error, @component_instance.invoke("test-callback"))
    #   assert_equal("Hello World!", @component_instance.invoke("test-callback", "World!", "Bu"))
    # end

    private

    def source
      <<~SLINT
        export global Glob {
          in-out property <int> my_global_property: 100;
          in-out property <float> global_float_property: 30.3;
          in-out property <string> global_text_prop: "global-string-value";
          in-out property <bool> global_bool_prop: false;
          in-out property <color> global_col_prop: #ffaaff;
        }

        export struct MyStruct {
          title: string,
          finished: bool,
          dash-prop: bool,
        }

        export component App inherits Window {
          in-out property <int> int_property: 42;
          in-out property <float> float_prop: 110;
          in-out property <string> text_prop: "test-string-value";
          in-out property <bool> bool_prop: true;
          in-out property <color> col_prop: #ffaaff;
          in-out property <MyStruct> struct_prop: {
            title: "builtin",
            finished: true,
            dash-prop: true,
          };
          // to be implemented
          // in-out property <image> some_image: @image-url("")

          callback test-callback(string) -> string;
          test-callback(value) => {
            return "Hello " + value;
          }
        }
      SLINT
    end
  end
end
