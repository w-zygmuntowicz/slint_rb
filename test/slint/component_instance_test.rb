# frozen_string_literal: true

require "test_helper"

module Slint
  class ComponentInstanceTest < Minitest::Test
    def test_definition
      compiler = Compiler.new
      compilation_result = compiler.build_from_source("export component App inherits Window {}", "")
      component_definition = compilation_result.components.first
      component_instance = component_definition.create

      # Workaround: no equality operator for component definition, we compare by name
      assert_equal(component_definition.name, component_instance.definition.name)
    end
  end
end
