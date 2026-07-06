# frozen_string_literal: true

module Slint
  # Ruby wrapper around slint_interpreter::component_instance
  class ComponentInstance
    def invoke(name, *args)
      r_invoke(name, args)
    end
  end
end
