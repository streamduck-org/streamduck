// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Threading.Tasks;
using Streamduck.Inputs;
using Streamduck.Interfaces;

namespace Streamduck.Triggers;

public abstract class Trigger : INamed {
	public abstract string? Description { get; }
	public abstract string Name { get; }
	public abstract bool IsApplicableTo(Input input);
	public abstract Task<TriggerInstance> CreateInstance();
}