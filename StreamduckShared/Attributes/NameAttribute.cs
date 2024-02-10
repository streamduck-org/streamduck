// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;

namespace Streamduck.Attributes;

/**
 * Renames property in UI, methods, parameters and return types. If name is empty, title will not be shown
 */
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field | AttributeTargets.Method
                | AttributeTargets.Parameter | AttributeTargets.ReturnValue)]
public class NameAttribute(string name) : Attribute {
	public string Name { get; } = name;
}