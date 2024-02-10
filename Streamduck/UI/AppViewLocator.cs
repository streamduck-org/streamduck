// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using NLog;
using ReactiveUI;

namespace Streamduck.UI;

public class AppViewLocator : IViewLocator {
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();

	public IViewFor? ResolveView<T>(T? viewModel, string? contract = null) {
		var viewModelName = viewModel!.GetType().FullName!;
		var viewTypeName = viewModelName.Replace("ViewModels", "Views")
			.TrimEnd("Model".ToCharArray());

		try {
			var viewType = Type.GetType(viewTypeName);
			if (viewType != null) return Activator.CreateInstance(viewType) as IViewFor;
			_l.Error($"Could not find the view {viewTypeName} for view model {viewModelName}.");
			return null;
		} catch (Exception) {
			_l.Error($"Could not instantiate view {viewTypeName}.");
			throw;
		}
	}
}