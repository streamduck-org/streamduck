using System.Text.Json.Serialization;

namespace Streamduck.Definitions.UI;

[JsonDerivedType(typeof(Header))]
[JsonDerivedType(typeof(StaticText))]
[JsonDerivedType(typeof(Label))]
[JsonDerivedType(typeof(StringInput))]
[JsonDerivedType(typeof(IntegerInput))]
[JsonDerivedType(typeof(NumberInput))]
[JsonDerivedType(typeof(Array))]
[JsonDerivedType(typeof(NestedFields))]
public abstract class FieldType {
	/**
	 * Object Type
	 */
	public abstract string Type { get; }

	/**
	 * Displays title text in a large text
	 */
	public class Header : FieldType {
		public override string Type => "Header";
	}

	/**
	 * Displays description text in a normal text font with optional title
	 */
	public class StaticText : FieldType {
		public override string Type => "StaticText";
	}

	/**
	 * Displays provided text or value if bound
	 */
	public class Label : FieldType {
		public override string Type => "Label";
		public string? Text { get; init; }
	}

	/**
	 * Input field that changes the bound value, can contain any UTF-8 character
	 */
	public class StringInput : FieldType {
		public override string Type => "StringInput";

		public bool Disabled { get; init; }
	}

	/**
	 * Input field that changes the bound value, can contain any whole number
	 */
	public class IntegerInput : FieldType {
		public override string Type => "IntegerInput";
		public bool Disabled { get; init; }
	}

	/**
	 * Input field that changes the bound value, can contain any real number
	 */
	public class NumberInput : FieldType {
		public override string Type => "NumberInput";
		public bool Disabled { get; init; }
	}

	public class Array : FieldType {
		public override string Type => "Array";
		public Field[] ElementSchema { get; init; } = System.Array.Empty<Field>();

		/**
		 * Serializable object that will be created when UI is trying to add an element to the array.
		 * If null, UI will not be able to create any elements in this array
		 */
		public object? NewElementTemplate { get; init; }

		public bool AllowRemoving { get; init; }

		public bool AllowReorder { get; init; }
	}

	public class NestedFields : FieldType {
		public override string Type => "NestedFields";
		public Field[] Schema { get; init; } = System.Array.Empty<Field>();

		/**
		 * If the nested fields should be inside of a collapsable menu
		 */
		public bool Collapsable { get; init; }
	}
}