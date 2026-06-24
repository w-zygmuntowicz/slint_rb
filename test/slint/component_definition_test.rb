# frozen_string_literal: true

require "test_helper"

module Slint
  class ComponentDefinitionTest < Minitest::Test
    def setup
      compiler = Compiler.new
      compilation_result = compiler.build_from_source(source, "")
      @component_definition = compilation_result.components.first
    end

    def test_create_when_valid
      component_instance = @component_definition.create

      assert_instance_of(ComponentInstance, component_instance)
    end

    def test_name
      assert_equal("MyAppName", @component_definition.name)
    end

    def test_callbacks
      assert_equal(["clicked"], @component_definition.callbacks)
    end

    def test_functions
      assert_equal(["my-fun"], @component_definition.functions)
    end

    def test_properties
      expected_properties = {
        "text-prop" => :string,
        "count-prop" => :number,
        "active-prop" => :bool
      }

      assert_equal(expected_properties, @component_definition.properties)
    end

    def test_globals
      assert_equal(["MyGlobal"], @component_definition.globals)
    end

    def test_global_properties
      expected = {
        "text-prop" => :string,
        "bool-prop" => :bool
      }

      assert_equal(expected, @component_definition.global_properties("MyGlobal"))
      assert_nil(@component_definition.global_properties("MyNonExistentGlobal"))
    end

    def test_gobal_callbacks
      assert_equal(["hello-callback"], @component_definition.global_callbacks("MyGlobal"))
      assert_nil(@component_definition.global_callbacks("MyNonExistentGlobal"))
    end

    private

    def source
      <<~SLINT
        export global MyGlobal  {
          in-out property<string> text-prop;
          in-out property<bool> bool-prop;

          callback hello-callback;
        }

        export component MyAppName inherits Window {
          callback clicked;

          public function my-fun() {}

          in-out property <string> text-prop;
          in-out property <int> count-prop;
          in-out property <bool> active-prop;
        }
      SLINT
    end
  end
end
