using System;
using System.Collections.Generic;
using System.Linq;
using System.Numerics;
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
		typeof(IgnoreAttribute),
		typeof(ReadOnlyAttribute),
		typeof(SwitchAttribute),
		typeof(CustomAttribute)
	};
	
	private static readonly List<Type> GenericAttributesToLookFor = new() {
		typeof(NumberFieldAttribute<>)
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

				foreach (var lookingFor in GenericAttributesToLookFor
					         .Where(lookingFor => attribute.GetType().IsGenericType 
					                              && attribute.GetType().GetGenericTypeDefinition() == lookingFor)) {
					receivedAttributes[attribute.GetType()] = attribute;
				}
			}

			// Handle Header first
			if (receivedAttributes.Cast<HeaderAttribute>() is { } headerAttribute) 
				yield return new Field.Header(headerAttribute.Text);
			
			// Handle static text
			foreach (var attribute in attributes) {
				if (attribute is StaticTextAttribute staticTextAttribute) {
					yield return new Field.StaticText(staticTextAttribute.Text);
				}
			}

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
				if (HasSetter(property)) {
					// Do text field instead
					string Getter() => (string)property.GetValue(obj)!;
					Action<string>? setter = IsReadWrite(property, receivedAttributes)
						? val => property.SetValue(obj, val)
						: null;

					yield return new Field.StringInput(title, Getter, setter) {
						Description = description
					};
				} else {
					// Do label
					yield return new Field.Label(title, () => (string)property.GetValue(obj)!) {
						Description = description
					};
				}
			}

			if (type.IsAssignableTo(typeof(INumber<>))) {
				yield return (Field)typeof(FieldReflector).GetMethod(
						nameof(MakeGenericNumberField), 
						BindingFlags.NonPublic | BindingFlags.Static
						)!.MakeGenericMethod(type)
					.Invoke(null, new object[] { property, receivedAttributes, obj, title, description! })!;
			}
		}
	}

	private static Field MakeGenericNumberField<T>(PropertyInfo property, IDictionary<Type, object> receivedAttributes, object obj, string title, string? description) where T : INumber<T> {
		var attr = Cast<NumberFieldAttribute<T>>(receivedAttributes)! ?? new NumberFieldAttribute<T>();
		Action<T>? setter = IsReadWrite(property, receivedAttributes)
			? val => property.SetValue(obj, val)
			: null;
		return new Field.NumberInput<T>(title, Getter, setter) {
			Description = description,
			Min = attr.Min,
			Max = attr.Max,
			EnforceLimit = attr.EnforceLimit,
			Slider = attr.Slider
		};
		T Getter() => (T)property.GetValue(obj)!;
	}

	private static bool HasSetter(PropertyInfo property) => property.GetSetMethod(true) != null;
	
	private static bool HasPublicSetter(PropertyInfo property) => property.GetSetMethod() != null;
	
	private static bool IsReadOnly(IDictionary<Type, object> receivedAttributes) =>
		receivedAttributes.ContainsKey(typeof(ReadOnlyAttribute));

	private static bool IsReadWrite(PropertyInfo property, IDictionary<Type, object> receivedAttributes) =>
		HasPublicSetter(property) && !IsReadOnly(receivedAttributes);

	private static T? Cast<T>(this IDictionary<Type, object> dictionary) where T : class {
		dictionary.TryGetValue(typeof(T), out var value);
		return value as T;
	}

	public static string FormatName(string name) => string.Concat(name
		.Select(x => char.IsUpper(x) ? $" {x}" : $"{x}")).TrimStart(' ');
}