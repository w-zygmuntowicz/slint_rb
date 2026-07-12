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

    def test_col_from_str
      col = Color.new("#123456")

      assert_equal(0x12, col.red)
      assert_equal(0x34, col.green)
      assert_equal(0x56, col.blue)
      assert_equal(255, col.alpha)
      assert_equal("argb(255, 18, 52, 86)", col.to_s)
    end

    def test_rgb_color_constructor
      col = Color.new(red: 0x12, green: 0x34, blue: 0x56)

      assert_equal(0x12, col.red)
      assert_equal(0x34, col.green)
      assert_equal(0x56, col.blue)
      assert_equal(255, col.alpha)
    end

    def test_col_from_rgba
      col = Color.new(red: 0x12, green: 0x34, blue: 0x56, alpha: 128)

      assert_equal(0x12, col.red)
      assert_equal(0x34, col.green)
      assert_equal(0x56, col.blue)
      assert_equal(128, col.alpha)
    end

    def test_comparison
      col1 = Color.new(red: 0x12, green: 0x34, blue: 0x56)
      col2 = Color.new(red: 0x12, green: 0x34, blue: 0x56)

      assert_equal(col1, col2)
    end

    def test_transparentize
      red = Color.new(alpha: 200, red: 255, green: 0, blue: 0)

      assert_equal(red.transparentize(0.5), Color.new(alpha: 100, red: 255, green: 0, blue: 0))
    end

    def test_mix
      mostly_red = Color.new(red: 200, green: 0, blue: 0)
      black = Color.new(red: 0, green: 0, blue: 0)

      assert_equal(Color.new(red: 100, green: 0, blue: 0), mostly_red.mix(black, 0.5))
    end

    def test_brighter_multiplies_hsv_value_by_one_plus_factor
      maroonish = Color.new(red: 100, green: 0, blue: 0)
      reddish = Color.new(red: 180, green: 0, blue: 0)

      assert_equal(reddish, maroonish.brighter(0.8))
    end

    def test_darker_divides_hsv_value_by_one_plus_factor
      reddish = Color.new(red: 130, green: 0, blue: 0)

      assert_equal(Color.new(red: 100, green: 0, blue: 0), reddish.darker(0.3))
    end

    def test_with_alpha
      red = Color.new(red: 255, green: 0, blue: 0)

      assert_equal(Color.new(alpha: 127, red: 255, green: 0, blue: 0), red.with_alpha(0.5))
    end
  end
end
