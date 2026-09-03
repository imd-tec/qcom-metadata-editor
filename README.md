# qcom-metadata-editor

Read or edit the enabled device tree overlays inside a Qualcomm `qclinux_fit.img`.

## Build

```sh
cargo build --release
```

## Usage

Show the current FIT configuration table:

```sh
qcom-metadata-editor qclinux_fit.img
```

Update configuration index 1 to load a different overlay list:

```sh
qcom-metadata-editor qclinux_fit.img -u -i 1 -f "fdt-qcs8550-imdt-sbc.dtb"
```

Run `qcom-metadata-editor --help` for the full list of options.
