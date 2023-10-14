using System;
using System.Linq;

namespace Streamduck.Utils; 

public static class UtilMethods {
	public static string FormatAsWords(this string name) => string.Concat(name
		.Select((x, i) => i == 0 ? $"{char.ToUpper(x)}" : char.IsUpper(x) ? $" {x}" : $"{x}")).TrimStart(' ');

	public static T? WeakToNullable<T>(this WeakReference<T> weakReference) where T : class {
		weakReference.TryGetTarget(out var result);
		return result;
	}
}