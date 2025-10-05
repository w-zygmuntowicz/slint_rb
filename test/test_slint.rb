# frozen_string_literal: true

require "test_helper"

class TestSlint < Minitest::Test
  def test_compiler
    path = "test/ui/app-window.slint"
    compiler = Slint::Compiler.new
    compiler.build_from_path(path)
  end
end
