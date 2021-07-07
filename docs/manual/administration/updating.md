---
title: Updating
description: How to update Vector to a newer version.
---

Updating Vector depends largely on your [installation][docs.installation]
method. Each installation guide provides its own "Updating" section:

## Installation Method

### Package managers

<Jump to="/docs/setup/installation/package-managers/apt/#updating">APT</Jump>
<Jump to="/docs/setup/installation/package-managers/dpkg/#updating">DPKG</Jump>
<Jump to="/docs/setup/installation/package-managers/homebrew/#updating">Homebrew</Jump>
<Jump to="/docs/setup/installation/package-managers/msi/#updating">MSI</Jump>
<Jump to="/docs/setup/installation/package-managers/nix/#updating">Nix</Jump>
<Jump to="/docs/setup/installation/package-managers/rpm/#updating">RPM</Jump>
<Jump to="/docs/setup/installation/package-managers/yum/#updating">YUM</Jump>

### Manual

<Jump to="/docs/setup/installation/manual/from-archives/#updating">Updating from archives</Jump>
<Jump to="/docs/setup/installation/manual/from-source/#updating">Updating from source</Jump>

## Working Upstream

Depending on your [topology][docs.topologies], you'll want update your Vector
instances in a specific order. You should _always_ start downstream and work
your way upstream. This allows for incremental updating across your topology,
ensuring downstream Vector instances do not receive data in formats that are
unrecognized. Vector always makes a best effort to successfully process data,
but there is no guarantee of this if a Vector instance is handling a data
format defined by a future unknown Vector version.

## Capacity Planning

When updating, you'll be taking Vector instances off-line for a short period of time,
and upstream data will accumulate and buffer. To avoid overloading your instances,
you'll want to make sure you have enough capacity to handle the surplus of
data. We recommend provisioning at least 20% of head room, on all resources,
to account for spikes and updating.

[docs.installation]: /docs/setup/installation/
[docs.topologies]: /docs/setup/deployment/topologies/
