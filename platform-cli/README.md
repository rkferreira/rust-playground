# Platform CLI tool

This is a simple CLI tool that allows you to interact with the Platform terraform modules.

## Build

To build the CLI tool you must have rust installed. You can install rust by running the following command:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installing rust you can build the CLI tool by running the following command:

```
git clone https://github.com/rkferreira/platform-cli.git
cd platform-cli
cargo build --release && sudo cp target/release/platform-cli /usr/local/bin \
    && sudo chmod +x /usr/local/bin/platform-cli

```

## Usage

This tool will guide you through the process of creating a new deployment, it's interative and will ask you for the necessary information to create a new deployment.

Create base path for your projects:

```
cd $HOME
mkdir -p my-projects/src
cd my-projects

```

### Init

Create a new deployment:

```
platform-cli init

```

This will create a folder with the provided system name and store configurations on:

```
src/<system_name>/.platform-config.json

```

### Plan

For this step to suceed you must have terraform binary installed on your machine and on current command PATH.

Running this command will generate a plan for the deployment, based on the config file you created before (init phase).

```
platform-cli plan

```

Result is stored on folder:

```
src/<system_name>/.cache/

```

It will contain terraform files and a plan file.


### Apply

This command will apply the plan generated on the previous step.

It will ask for confirmation before applying the plan.

```
platform-cli apply

```

Calling terraform under the hood, it will apply the plan and create the resources on the cloud provider.

