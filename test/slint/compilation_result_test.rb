# frozen_string_literal: true

require "test_helper"

module Slint
  class CompilationResultTest < Minitest::Test
    def setup
      @compiler = Compiler.new
    end

    def test_valid_when_valid
      compilation_result = @compiler.build_from_source("export component App {}", "")

      assert(compilation_result.valid?)
    end

    def test_valid_when_invalid
      compilation_result = @compiler.build_from_source("export component App }", "")

      assert_equal(false, compilation_result.valid?)
    end
  end
end
