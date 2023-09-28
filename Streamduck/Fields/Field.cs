using System;
using System.Numerics;

namespace Streamduck.Fields;

public abstract class Field {
	protected Field(string title) {
		Title = title;
	}
	
	public string Title { get; }
	public string? Description { get; init; }
	
	/**
	 * Displays title text in a large text
	 */
	public class Header : Field {
		public Header(string title) : base(title) { }
	}

	/**
	 * Displays description text in a normal text font with optional title
	 */
	public class StaticText : Field {
		public StaticText(string title) : base(title) { }
	}

	/**
	 * Displays text from text accessor
	 */
	public class Label : Field {
		private readonly Func<string> _getter;
		
		public Label(string title, Func<string> textGetter) : base(title) {
			_getter = textGetter;
		}

		public string Text => _getter.Invoke();
	}

	/**
	 * Checkbox
	 */
	public class Checkbox : Field {
		public Checkbox(string title) : base(title) { }
	}
	
	/**
	 * Horizontal switch
	 */
	public class Switch : Field {
		public Switch(string title) : base(title) { }
	}

	/**
	 * Input field that changes the bound value, can contain any UTF-8 character
	 */
	public class StringInput : Field {
		private readonly Func<string> _getter;
		private readonly Action<string>? _setter;

		public bool Disabled { get; }
		
		public StringInput(string title, Func<string> getter, Action<string>? setter = null) : base(title) {
			_getter = getter;
			_setter = setter;
			Disabled = _setter == null;
		}

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
		
		public bool Disabled { get; }
		public bool Slider { get; init; }
		public bool EnforceLimit { get; init; }
		
		public T? Min { get; init; }
		public T? Max { get; init; }
		
		public NumberInput(string title, Func<T> getter, Action<T>? setter = null) : base(title) {
			_getter = getter;
			_setter = setter;
			Disabled = _setter == null;
		}

		public T Value {
			get => _getter.Invoke();
			set => _setter?.Invoke(value);
		}
	}

	public class Array : Field {
		public Field[] ElementSchema { get; init; } = System.Array.Empty<Field>();

		/**
		 * Serializable object that will be created when UI is trying to add an element to the array.
		 * If null, UI will not be able to create any elements in this array
		 */
		public object? NewElementTemplate { get; init; }

		public bool AllowRemoving { get; init; }

		public bool AllowReorder { get; init; }
		public Array(string title) : base(title) { }
	}

	public class NestedFields : Field {
		public Field[] Schema { get; init; } = System.Array.Empty<Field>();

		/**
		 * If the nested fields should be inside of a collapsable menu
		 */
		public bool Collapsable { get; init; }

		public NestedFields(string title) : base(title) { }
	}
}