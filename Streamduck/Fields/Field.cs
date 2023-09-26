using System;

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
		public Label(string title, Func<string> textAccessor) : base(title) {
			TextAccessor = textAccessor;
		}
		
		public Func<string> TextAccessor { get; }
	}

	/**
	 * Input field that changes the bound value, can contain any UTF-8 character
	 */
	public class StringInput : Field {

		public bool Disabled { get; init; }
		public StringInput(string title) : base(title) { }
	}

	/**
	 * Input field that changes the bound value, can contain any whole number
	 */
	public class IntegerInput : Field {
		public bool Disabled { get; init; }
		public IntegerInput(string title) : base(title) { }
	}

	/**
	 * Input field that changes the bound value, can contain any real number
	 */
	public class NumberInput : Field {
		public bool Disabled { get; init; }
		public NumberInput(string title) : base(title) { }
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