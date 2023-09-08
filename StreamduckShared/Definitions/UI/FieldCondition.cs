using System;
using System.Text.Json.Serialization;

namespace Streamduck.Definitions.UI;

[JsonDerivedType(typeof(Always))]
[JsonDerivedType(typeof(Never))]
[JsonDerivedType(typeof(Exists))]
[JsonDerivedType(typeof(EqualTo))]
[JsonDerivedType(typeof(Contains))]
[JsonDerivedType(typeof(RegexMatches))]
[JsonDerivedType(typeof(GreaterThan))]
[JsonDerivedType(typeof(GreaterThanOrEquals))]
[JsonDerivedType(typeof(LesserThan))]
[JsonDerivedType(typeof(LesserThanOrEquals))]
[JsonDerivedType(typeof(Not))]
[JsonDerivedType(typeof(Or))]
[JsonDerivedType(typeof(And))]
public abstract class FieldCondition {
	/**
	 * Object Type
	 */
	public abstract string Type { get; }

	/**
	 * Field is always visible
	 */
	public class Always : FieldCondition {
		public override string Type => "Always";
	}

	/**
	 * Field is never visible
	 */
	public class Never : FieldCondition {
		public override string Type => "Never";
	}

	/**
	 * Field will be shown if a certain path exists in the state
	 */
	public class Exists : FieldCondition {
		public override string Type => "Exists";
		public string[] ValuePath { get; init; } = Array.Empty<string>();
	}

	/**
	 * Field will be shown if path has the same value as specified
	 */
	public class EqualTo : FieldCondition {
		public override string Type => "EqualTo";
		public string[] ValuePath { get; init; } = Array.Empty<string>();
		public object? ExpectedValue { get; init; }
	}

	/**
	 * Field will be shown if path contains specified value, example "hello" in "hello world"
	 */
	public class Contains : FieldCondition {
		public override string Type => "Contains";
		public string[] ValuePath { get; init; } = Array.Empty<string>();
		public string ContainsValue { get; init; } = "";
	}

	/**
	 * Field will be shown if path's value matches the regex pattern, should use ECMAScript flavor of Regex
	 */
	public class RegexMatches : FieldCondition {
		public override string Type => "RegexMatches";
		public string[] ValuePath { get; init; } = Array.Empty<string>();
		public string Pattern { get; init; } = "";

		/**
		 * Regex flags that should be used, example "gm"
		 * <list type="bullet">
		 *     <item>
		 *         <description>g - Global search</description>
		 *     </item>
		 *     <item>
		 *         <description>i - Case-insensitive search</description>
		 *     </item>
		 *     <item>
		 *         <description>m - Allows `^` and `$` to match newline characters</description>
		 *     </item>
		 *     <item>
		 *         <description>s - Allows `.` to match newline characters</description>
		 *     </item>
		 *     <item>
		 *         <description>u - Unicode</description>
		 *     </item>
		 *     <item>
		 *         <description>y - Sticky search</description>
		 *     </item>
		 * </list>
		 */
		public string Flags { get; init; } = "";
	}

	/**
	 * Field will be shown if value at the path will have value greater than specified
	 */
	public class GreaterThan : FieldCondition {
		public override string Type => "GreaterThan";
		public string[] ValuePath { get; init; } = Array.Empty<string>();
		public object? Value { get; init; }
	}

	/**
	 * Field will be shown if value at the path will have value equal or greater than specified
	 */
	public class GreaterThanOrEquals : FieldCondition {
		public override string Type => "GreaterThanOrEquals";
		public string[] ValuePath { get; init; } = Array.Empty<string>();
		public object? Value { get; init; }
	}

	/**
	 * Field will be shown if value at the path will have value lesser than specified
	 */
	public class LesserThan : FieldCondition {
		public override string Type => "LesserThan";
		public string[] ValuePath { get; init; } = Array.Empty<string>();
		public object? Value { get; init; }
	}

	/**
	 * Field will be shown if value at the path will have value equal or lesser than specified
	 */
	public class LesserThanOrEquals : FieldCondition {
		public override string Type => "LesserThanOrEquals";
		public string[] ValuePath { get; init; } = Array.Empty<string>();
		public object? Value { get; init; }
	}

	/**
	 * Field will be shown if condition inside is false
	 */
	public class Not : FieldCondition {
		public override string Type => "Not";
		public FieldCondition Condition { get; init; } = new Always();
	}

	/**
	 * Field will be shown if any condition inside is true
	 */
	public class Or : FieldCondition {
		public override string Type => "Or";
		public FieldCondition[] Conditions { get; init; } = Array.Empty<FieldCondition>();
	}

	/**
	 * Field will be shown if all conditions inside are true
	 */
	public class And : FieldCondition {
		public override string Type => "And";
		public FieldCondition[] Conditions { get; init; } = Array.Empty<FieldCondition>();
	}
}