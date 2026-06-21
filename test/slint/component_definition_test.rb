# frozen_string_literal: true

require "test_helper"

module Slint
  class ComponentDefinitionTest < Minitest::Test
    def setup
      @compiler = Compiler.new
    end

    def test_create_when_valid
      compilation_result = @compiler.build_from_source("export component App inherits Window {}", "")
      component_definition = compilation_result.components.first

      component_instance = component_definition.create

      assert_instance_of(ComponentInstance, component_instance)
    end

    def test_name
      compilation_result = @compiler.build_from_source("export component MyAppName inherits Window {}", "")
      component_definition = compilation_result.components.first

      assert_equal("MyAppName", component_definition.name)
    end

    def test_callbacks
      compilation_result = @compiler.build_from_source("export component App inherits Window { callback clicked; }", "")
      component_definition = compilation_result.components.first

      assert_equal(["clicked"], component_definition.callbacks)
    end
  end
end
