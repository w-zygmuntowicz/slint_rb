# frozen_string_literal: true

require "test_helper"

module Slint
  class DiagnosticsTest < Minitest::Test
    def setup
      compiler = Compiler.new
      @compilation_result = compiler.build_from_source("export component App }", "")
      @diagnostic = @compilation_result.diagnostics.first
    end

    def test_level
      assert_equal(:error, @diagnostic.level)
    end

    def test_message
      assert_instance_of(String, @diagnostic.message)
    end

    def test_line_column
      assert_instance_of(Array, @diagnostic.line_column)
      assert_equal(2, @diagnostic.line_column.length)
    end

    def test_source_file
      assert_instance_of(String, @diagnostic.source_file)
    end
  end
end
