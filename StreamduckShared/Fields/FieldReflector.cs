using System;
using System.Collections.Generic;
using System.Linq;
using System.Numerics;
using System.Reflection;
using Streamduck.Fields.Attributes;

namespace Streamduck.Fields;

/**
 * Class that analyzes objects for fields
 */
public static partial class FieldReflector {
	private const BindingFlags Flags = BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance;

	public static IEnumerable<Field> AnalyzeObject(object? obj) {
		if (obj == null) {
			yield return new Field.StaticText("Object was null");
			yield break;
		}
		
		foreach (var property in obj.GetType().GetProperties(Flags)) {
			// Handle Header first
			if (property.GetCustomAttribute<HeaderAttribute>() is { } headerAttribute) 
				yield return new Field.Header(headerAttribute.Text);
			
			// Handle static text
			foreach (var attribute in property.GetCustomAttributes<StaticTextAttribute>()) {
				yield return new Field.StaticText(attribute.Text);
			}

			// Property must have a getter to be considered for UI
			var getMethod = property.GetGetMethod(true);
			if (getMethod is null) continue;

			// Continue if not public and doesn't have Include attribute
			if ((!getMethod.IsPublic && property.GetCustomAttribute<IncludeAttribute>() == null) 
			    || property.GetCustomAttribute<IgnoreAttribute>() != null) continue;

			var title = GetMemberName(property);
			var description = GetMemberDescription(property);
			
			var type = property.PropertyType;

			if (type == typeof(string)) {
				// Check if there's public setter
				if (HasSetter(property)) {
					// Do text field instead
					string Getter() => (string)property.GetValue(obj)!;
					Action<string>? setter = IsReadWrite(property)
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
			} else if (type.IsAssignableTo(typeof(INumber<>))) {
				yield return (Field)typeof(FieldReflector).GetMethod(
						nameof(MakeGenericNumberField), 
						BindingFlags.NonPublic | BindingFlags.Static
						)!.MakeGenericMethod(type)
					.Invoke(null, new[] { property, obj, title, description! })!;
			} else if (type == typeof(bool)) {
				bool Getter() => (bool)property.GetValue(obj)!;
				Action<bool>? setter = IsReadWrite(property)
					? val => property.SetValue(obj, val)
					: null;

				yield return new Field.Checkbox(title, Getter, setter) {
					Description = description,
					SwitchStyle = property.GetCustomAttribute<SwitchAttribute>() != null
				};
			} else if (type.IsAssignableTo(typeof(Enum))) {
				var variants = type.GetFields(BindingFlags.Public | BindingFlags.Static);
				var underlyingType = type.GetEnumUnderlyingType();
				if (property.GetCustomAttribute<BitmaskAttribute>() != null) {
					yield return (Field)typeof(FieldReflector).GetMethod(
							nameof(MakeGenericBitmask), 
							BindingFlags.NonPublic | BindingFlags.Static
						)!.MakeGenericMethod(underlyingType)
						.Invoke(null, new[] { property, obj, variants, title, description! })!;
				} else {
					string FindVariant() {
						var value = property.GetValue(obj)!;

						foreach (var variant in variants) {
							if (value.Equals(variant.GetValue(null))) {
								return GetMemberName(variant);
							}
						}

						return "Unknown";
					}

					void SetVariant(string variant) {
						foreach (var variantInfo in variants) {
							var name = GetMemberName(variantInfo);
							
							if (name.Equals(variant)) {
								property.SetValue(obj, variantInfo.GetValue(null));
							}
						}
					}
					
					string Getter() => FindVariant();
					Action<string>? setter = IsReadWrite(property)
						? SetVariant
						: null;

					yield return new Field.Choice(title, Getter, setter, HumanReadableVariants(variants).ToArray()) {
						Description = description
					};
				}
			} else if (type.IsClass && Nullable.GetUnderlyingType(type) == null) { // Use recursion for anything else
				yield return new Field.NestedFields(title) {
					Description = description,
					Schema = AnalyzeObject(property.GetValue(obj)).ToArray()
				};
			}
			
			// TODO: Split all field creation into private methods and implement collection support 
		}
	}

	private static Field MakeGenericNumberField<T>(PropertyInfo property, object obj, string title, string? description) 
		where T : INumber<T> {
		var attr = property.GetCustomAttribute<NumberFieldAttribute<T>>() ?? new NumberFieldAttribute<T>();
		Action<T>? setter = IsReadWrite(property)
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

	private static Field MakeGenericBitmask<T>(PropertyInfo property, object obj, FieldInfo[] variants, string title, string? description)
		where T : IBinaryInteger<T> {
		return new Field.MultiChoice(
			title,
			() => GetVariantStatuses().ToArray(),
			SetVariantStatus,
			HumanReadableVariants(variants).ToArray()
		) {
			Description = description
		};

		long SetFlagAny(FieldInfo variantInfo, bool state) {
			var currentValue = Convert.ToInt64(property.GetValue(obj));
			var variantValue = Convert.ToInt64(variantInfo.GetValue(null));

			return state
				? currentValue | variantValue
				: currentValue & ~variantValue;
		}
		
		ulong SetFlagULong(FieldInfo variantInfo, bool state) {
			var currentValue = Convert.ToUInt64(property.GetValue(obj));
			var variantValue = Convert.ToUInt64(variantInfo.GetValue(null));

			return state
				? currentValue | variantValue
				: currentValue & ~variantValue;
		}

		void SetVariantStatus(string variant, bool state) {
			foreach (var variantInfo in variants) {
				var name = GetMemberName(variantInfo);

				if (!name.Equals(variant)) continue;

				property.SetValue(
					obj, 
					typeof(T) == typeof(ulong) 
						? Enum.ToObject(property.PropertyType, SetFlagULong(variantInfo, state))
						: Enum.ToObject(property.PropertyType, SetFlagAny(variantInfo, state))
				);
			}
		}

		IEnumerable<bool> GetVariantStatuses() {
			var value = (Enum)property.GetValue(obj)!;
			
			foreach (var variant in variants) {
				var variantValue = (Enum)variant.GetValue(null)!;
				var hasFlag = value.HasFlag(variantValue);
				
				yield return hasFlag;
			}
		}
	}

	private static bool HasSetter(PropertyInfo property) => property.GetSetMethod(true) != null;
	
	private static bool HasPublicSetter(PropertyInfo property) => property.GetSetMethod() != null;

	private static bool IsReadWrite(PropertyInfo property) =>
		(HasPublicSetter(property) || (property.GetCustomAttribute<IncludeAttribute>()?.WriteAllowed ?? false)) 
		&& property.GetCustomAttribute<ReadOnlyAttribute>() == null;
	
	private static string GetMemberName(MemberInfo member) => 
		member.GetCustomAttribute<NameAttribute>()?.Name 
		?? FormatName(member.Name);

	private static string? GetMemberDescription(MemberInfo member) =>
		member.GetCustomAttribute<DescriptionAttribute>()?.Description;
	
	private static IEnumerable<(string, string?)> HumanReadableVariants(IEnumerable<FieldInfo> variants) =>
		from variant in variants 
		let name = GetMemberName(variant) 
		let variantDescription = GetMemberDescription(variant)
		select (name, variantDescription);

	public static string FormatName(string name) => string.Concat(name
		.Select(x => char.IsUpper(x) ? $" {x}" : $"{x}")).TrimStart(' ');
}