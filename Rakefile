# frozen_string_literal: true

require "bundler/gem_tasks"
require "minitest/test_task"
require "rubocop/rake_task"
require "rb_sys/extensiontask"

Minitest::TestTask.create
RuboCop::RakeTask.new

desc "Compiles rust extension"
task build: :compile

GEMSPEC = Gem::Specification.load("slint_rb.gemspec")

RbSys::ExtensionTask.new("slint_rb", GEMSPEC) do |ext|
  ext.lib_dir = "lib/slint_rb"
end

task default: %i[compile:dev test rubocop]
