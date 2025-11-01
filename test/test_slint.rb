# frozen_string_literal: true

require "test_helper"

class TestSlint < Minitest::Test
  def test_compiler
    path = "test/ui/app-window.slint"
    compiler = Slint::Compiler.new
    compilation_result = compiler.build_from_path(path)
    compilation_result.render
    binding.irb
  end
end
