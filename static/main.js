import uPlot from '/static/uPlot.esm.js';

export function main() {
    let plots = [{
        params: {
            probe: 'CMSIS-DAP',
            chip: 'nRF51822',
        }
    },
    {
        params: {
            probe: 'J-Link',
            chip: 'nRF52840',
        }
    }
    ];

    m.mount(document.body, {
        view(vnode) {
            return m('.container', [
                m('.row', plots.map((p, i) => m('.col-6.plot', {
                    oninit(vnode) {
                        vnode.state.plot = undefined;
                        vnode.state.resize = function () { };
                    },
                    oncreate(vnode) {
                        createPlot(plots[i], vnode.dom).then(plot => {
                            vnode.state.plot = plot;

                            vnode.state.resize = function () {
                                if (plot) {
                                    plot.setSize({
                                        width: vnode.dom.clientWidth,
                                        height: vnode.dom.clientWidth / 1.41,
                                    })
                                }
                            }

                            window.addEventListener('resize', vnode.state.resize)
                        });
                    },
                    onremove(vnode) {
                        window.removeEventListener('resize', vnode.state.resize)
                    }
                }))
                )
            ])
        }
    })
}

function createPlot(plot, dom) {
    return m.request({
        method: 'GET',
        url: '/list',
        params: plot.params
    }).then(function (result) {
        let data = [
            result.logs.map(
                (v, i) => i),
            result.logs.map(v => v.read_speed),
            result.logs.map(v => v.write_speed)
        ];
        const opts = {
            width: dom.clientWidth,
            height: dom.clientWidth / 1.41,
            title: plot.params.probe + ': ' + plot.params.chip,
            axes: [
                {
                    values: (self, vals) => vals.map(i => result.logs[i].commit_hash),
                    space: 100,
                    rotate: -90,
                    size: 100,
                }, {
                    values: (self, vals) => vals.map(i => i),
                    label: 'Transfer Speed [Bytes]'
                }
            ],
            scales: {
                x: {
                    date: false,
                    distr: 2,
                }
            },
            series: [
                {
                    label: 'commit',
                    value: (self, i) => result.logs[i].commit_hash
                }, {
                    stroke: 'orange',
                    label: 'read',
                    value: (self, i) => i
                }, {
                    stroke: 'red',
                    label: 'write',
                    value: (self, i) => i
                }
            ],
        };
        return new uPlot(opts, data, dom);
    });
}