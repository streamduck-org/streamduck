// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Numerics;

namespace Streamduck.Fields;

public abstract class Field(string title) {
	public string Title { get; } = title;
	public string? Description { get; init; }

	/**
	 * Displays title text in a large text
	 */
	public class Header(string title) : Field(title);

	/**
	 * Displays description text in a normal text font with optional title
	 */
	public class StaticText(string title) : Field(title);

	/**
	 * Displays text from text accessor
	 */
	public class Label(string title, Func<string> textGetter) : Field(title) {
		public string Text => textGetter.Invoke();
	}

	/**
	 * Checkbox
	 */
	public class Checkbox : Field {
		private readonly Func<bool> _getter;
		private readonly Action<bool>? _setter;

		public Checkbox(string title, Func<bool> getter, Action<bool>? setter) : base(title) {
			_getter = getter;
			_setter = setter;
			Disabled = _setter == null;
		}

		public bool Disabled { get; }
		public bool SwitchStyle { get; init; }

		public bool Value {
			get => _getter.Invoke();
			set => _setter?.Invoke(value);
		}
	}

	/**
	 * Input field that changes the bound value, can contain any UTF-8 character
	 */
	public class StringInput : Field {
		private readonly Func<string> _getter;
		private readonly Action<string>? _setter;

		public StringInput(string title, Func<string> getter, Action<string>? setter = null) : base(title) {
			_getter = getter;
			_setter = setter;
			Disabled = _setter == null;
		}

		public bool Disabled { get; }

		public string Value {
			get => _getter.Invoke();
			set => _setter?.Invoke(value);
		}
	}

	/**
	 * Input field that changes the bound value, can contain any number
	 */
	public class NumberInput<T> : Field where T : INumber<T> {
		private readonly Func<T> _getter;
		private readonly Action<T>? _setter;

		public NumberInput(string title, Func<T> getter, Action<T>? setter = null) : base(title) {
			_getter = getter;
			_setter = setter;
			Disabled = _setter == null;
		}

		public bool Disabled { get; }
		public bool Slider { get; init; }
		public bool EnforceLimit { get; init; }

		public T? Min { get; init; }
		public T? Max { get; init; }

		public T Value {
			get => _getter.Invoke();
			set => _setter?.Invoke(value);
		}
	}

	public class Choice : Field {
		private readonly Func<string> _getter;
		private readonly Action<string>? _setter;

		public Choice(string title, Func<string> getter, Action<string>? setter, (string, string?)[] variants) :
			base(title) {
			_getter = getter;
			_setter = setter;
			Variants = variants;
			Disabled = _setter == null;
		}

		public bool Disabled { get; }
		public (string, string?)[] Variants { get; }

		public string Value {
			get => _getter.Invoke();
			set => _setter?.Invoke(value);
		}
	}

	public class MultiChoice : Field {
		private readonly Func<bool[]> _getter;
		private readonly Action<string, bool>? _setter;

		public MultiChoice(string title, Func<bool[]> getter, Action<string, bool>? setter,
			(string, string?)[] variants) : base(title) {
			_getter = getter;
			_setter = setter;
			Variants = variants;
			Disabled = _setter == null;
		}

		public bool Disabled { get; }
		public (string, string?)[] Variants { get; }

		public bool? this[string name] {
			get {
				var index = FindVariantIndex(name);
				if (index < 0 || index >= Variants.Length) return null;
				return _getter.Invoke()[index];
			}

			set {
				if (_setter == null) return;
				if (value == null) return;

				var index = FindVariantIndex(name);
				if (index == -1) return;
				_setter.Invoke(name, value.Value);
			}
		}

		public bool[] Values => _getter.Invoke();

		private int FindVariantIndex(string name) {
			for (var i = 0; i < Variants.Length; i++) {
				if (Variants[i].Item1.Equals(name)) return i;
			}

			return -1;
		}
	}

	public class Array(string title, Field[][] values) : Field(title) {
		public Field[][] Values { get; init; } = values;

		/**
		 * Serializable object that will be created when UI is trying to add an element to the array.
		 * If null, UI will not be able to create any elements in this array
		 */
		public object? NewElementTemplate { get; init; }

		public bool AllowRemoving { get; init; }

		public bool AllowReorder { get; init; }
	}

	public class NestedFields(string title) : Field(title) {
		public Field[] Schema { get; init; } = System.Array.Empty<Field>();
	}
}