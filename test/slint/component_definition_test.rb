# frozen_string_literal: true

require "test_helper"

module Slint
  class ComponentDefinitionTest < Minitest::Test
    def setup
      @compiler = Compiler.new
    end

    def test_create_when_valid
      compiler = Compiler.new
      compilation_result = compiler.build_from_source("export component App inherits Window {}", "")
      component_definition = compilation_result.components.first

      component_instance = component_definition.create

      assert_instance_of(ComponentInstance, component_instance)
    end
  end
end
