// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Threading.Tasks;

namespace Streamduck.Plugins.Methods;

public class ReflectedAction<T>(
	string name,
	Func<T, Task> actionToCall,
	string? description = null)
	: PluginAction<T> where T : class, new() {
	public override string Name { get; } = name;
	public override string? Description { get; } = description;
	public override Task Invoke(T data) => actionToCall.Invoke(data);
}