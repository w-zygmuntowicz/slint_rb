# frozen_string_literal: true

module Slint
  # Ruby wrapper around slint_interpreter::compiler
  class Compiler
    alias native_library_paths= library_paths=

    def library_paths=(paths)
      self.native_library_paths = paths.transform_keys { |library_name| String(library_name) }
    end
  end
end
