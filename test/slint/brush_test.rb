# frozen_string_literal: true

require "test_helper"

module Slint
  class BrushTest < Minitest::Test
    def test_default_color_constructor
      col = Color.new

      assert_equal(0, col.red)
      assert_equal(0, col.green)
      assert_equal(0, col.blue)
      assert_equal(0, col.alpha)
    end
  end
end
