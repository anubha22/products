# Logs

For an overview of this repository and the services in it, go to [the top-level README](../../README.md).
To view logs from deployed services, follow this guide.

We have the following types of logs:

- [Prometheus logs](#prometheus-logs)
- [Container logs](#container-logs)

More background on the infrastructure as a whole can be found at [`./infrastructure.md`](./infrastructure.md).

## Prometheus Logs

[Prometheus][prometheus] gathers logs from a [Kubernetes][kubernetes] cluster. It runs as a kubernetes pod automatically included with [Istio][istio], which we use to manage ingress and egress from our [Kubernetes][kubernetes] cluster.

To view the logs, use one of the following means:

- [local Prometheus dashboard](#viewing-a-cluster-prometheus-dashboard-locally)
- [Azure Log Analytics workspace](#viewing-prometheus-logs-in-azure)

For more on getting started with Prometheus, [go to the Prometheus docs][prometheus getting started], or watch [this introductory talk from Prometheus co-founder Julius Volz][prometheus co-founder video].

### Viewing a cluster Prometheus dashboard locally

You will need to target a specific kubernetes cluster for which you'd like to see logs.
See our [docs on connection to a Kubernetes cluster](./kubernetes.md#connecting-to-a-kubernetes-cluster).

Once you're targeting a Kubernetes cluster, run the following command to see its logs.

```
istioctl dashboard prometheus
```

### Viewing Prometheus logs in Azure

Alternatively, to see [Prometheus][prometheus] logs, you can run a query in the Azure Log Analytics workspace.

1. Open the "Log Analytics workspaces" area in the Azure portal.
2. Open the analytics workspace which is in the same resource group as your cluster.
3. Click "Logs" in the sidebar.
4. Run the following the query:

```
InsightsMetrics
| where Namespace == "prometheus"
```

## Container logs

### Using stern

To view all of the logs for the doc-index-updater:

```sh
stern -n doc-index-updater doc-index-updater
```

A useful command for viewing the logs related to a specific correlation id
would be to use the `-i` or `--include` parameter with a regex
which looks for `INFO`, `WARN`, and `ERROR` messages matching that id.

See here, for `b9912c04-325e-4f1b-8943-3ef319a88989`:

```sh
stern doc-index-updater -n doc-index-updater -i 'level.:.(INFO|WARN|ERROR).*b9912c04-325e-4f1b-8943-3ef319a88989'
```

See the [stern docs][stern] for more info.

[1]: ../scripts/update-kubernetes-config.sh
[stern]: https://github.com/wercker/stern
[kubernetes]: https://kubernetes.io/
[istio]: https://istio.io/
[prometheus]: https://prometheus.io/
[prometheus getting started]: https://prometheus.io/docs/prometheus/latest/getting_started/#starting-prometheus
[prometheus co-founder video]: https://www.youtube.com/watch?v=PDxcEzu62jk
