# frozen_string_literal: true

require "test_helper"

module Slint
  class CompilationResultTest < Minitest::Test
    def setup
      @compiler = Compiler.new
    end

    def test_valid_when_valid
      compilation_result = @compiler.build_from_source("export component App inherits Window {}", "")

      assert(compilation_result.valid?)
    end

    def test_valid_when_invalid
      compilation_result = @compiler.build_from_source("export component App }", "")

      refute(compilation_result.valid?)
    end

    def test_diagnostics_when_valid
      compilation_result = @compiler.build_from_source("export component App inherits Window {}", "")

      assert_empty(compilation_result.diagnostics)
    end

    def test_diagnostics_when_invalid
      compilation_result = @compiler.build_from_source("export component App }", "")

      refute_empty(compilation_result.diagnostics)
    end
  end
end
