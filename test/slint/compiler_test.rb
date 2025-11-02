# frozen_string_literal: true

require "test_helper"

module Slint
  class CompilerTest < Minitest::Test
    def test_include_paths_accessor
      compiler = Compiler.new

      assert_equal([], compiler.include_paths)

      compiler.include_paths = ["/path/one", "/path/two", "/path/three"]
      assert_equal(["/path/one", "/path/two", "/path/three"], compiler.include_paths)
    end
  end
end
