using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Text.RegularExpressions;
using Streamduck.Fields.Attributes;

namespace Streamduck.Fields;

/**
 * Class that analyzes objects for fields
 */
public static partial class FieldReflector {
	private const BindingFlags Flags = BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance;

	private static readonly List<Type> AttributesToLookFor = new() {
		typeof(HeaderAttribute),
		typeof(NameAttribute),
		typeof(DescriptionAttribute),
		typeof(IncludeAttribute),
		typeof(IgnoreAttribute)
	};

	public static IEnumerable<Field> AnalyzeObject(object obj) {
		foreach (var property in obj.GetType().GetProperties(Flags)) {
			var attributes = property.GetCustomAttributes(true);
			var receivedAttributes = new Dictionary<Type, object>();
			
			// Fetch all attributes right off the bat
			foreach (var attribute in attributes) {
				foreach (var lookingFor in AttributesToLookFor
					         .Where(lookingFor => attribute.GetType().IsAssignableTo(lookingFor))) {
					receivedAttributes[lookingFor] = attribute;
				}
			}

			// Handle Header first
			if (receivedAttributes.Cast<HeaderAttribute>() is { } headerAttribute) 
				yield return new Field.Header(headerAttribute.Text);

			// Property must have a getter to be considered for UI
			var getMethod = property.GetGetMethod(true);
			if (getMethod is null) continue;

			// Continue if not public and doesn't have Include attribute
			if ((!getMethod.IsPublic && !receivedAttributes.ContainsKey(typeof(IncludeAttribute))) 
			    || receivedAttributes.ContainsKey(typeof(IgnoreAttribute))) continue;

			var title = receivedAttributes.Cast<NameAttribute>()?.Name ?? FormatName(property.Name);
			var description = receivedAttributes.Cast<DescriptionAttribute>()?.Description;
			
			var type = property.PropertyType;

			if (type == typeof(string)) {
				// Check if there's public setter
				if (property.GetSetMethod(true)?.IsPublic ?? false) {
					// Do text field instead
				} else {
					// Do label
					yield return new Field.Label(title, () => (string)property.GetValue(obj)!);
				}
			}
		}
	}

	private static T? Cast<T>(this IDictionary<Type, object> dictionary) where T : class {
		dictionary.TryGetValue(typeof(T), out var value);
		return value as T;
	}

	public static string FormatName(string name) => string.Concat(name
		.Select(x => char.IsUpper(x) ? $" {x}" : $"{x}")).TrimStart(' ');
}