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

    def test_library_paths_accessor
      compiler = Compiler.new

      assert_equal({}, compiler.library_paths)

      compiler.library_paths = {
        "libfile.slint" => "third_party/libfoo/ui/lib.slint",
        "libdir" => "third_party/libbar/ui/"
      }
      assert_equal({
                     "libfile.slint" => "third_party/libfoo/ui/lib.slint",
                     "libdir" => "third_party/libbar/ui/"
                   }, compiler.library_paths)
    end

    def test_library_paths_setter_with_symbols
      compiler = Compiler.new

      compiler.library_paths = { library: "/path" }
      assert_equal({ "library" => "/path" }, compiler.library_paths)
    end
  end
end
