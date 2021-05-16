// vim: set filetype=groovy:

def image_name = "registry.finn-thorben.me/pixelflut"
def implementation = "rust"

pipeline {
    agent {
        kubernetes {
            yaml """
kind: Pod
spec:
  containers:
    - name: kustomize
      image: docker.io/nekottyo/kustomize-kubeval
      tty: true
      command:
      - cat
    - name: podman
      image: quay.io/podman/stable
      tty: true
      securityContext:
        privileged: true
      command:
        - cat
"""
        }
    }
    options {
        skipDefaultCheckout(true)
    }
    stages {
        stage("Checkout SCM") {
            steps {
                checkout scm
            }
        }
        stage("Build Kustomization") {
            steps {
                container("kustomize") {
                    gitStatusWrapper(
                        credentialsId: 'github-access',
                        description: 'Builds the main kustomization.yml to check for valid references and syntax',
                        failureDescription: 'kustomization.yml is not valid',
                        successDescription: 'kustomization.yml is valid',
                        gitHubContext: 'build-kustomization') {
                        sh "kustomize build . > k8s.yml"
                    }
                }
            }
        }
        stage("Check Kubernetes Config validity") {
            steps {
                container("kustomize") {
                    gitStatusWrapper(
                        credentialsId: 'github-access',
                        description: 'Validates the generated kubernetes config',
                        failureDescription: 'kubernetes config is not valid',
                        successDescription: 'kubernetes config is valid',
                        gitHubContext: 'check-k8s'
                    ) {
                        sh "kubeval k8s.yml --strict"
                    }
                }
            }
        }
        stage("Build Container Image") {
            steps {
                container("podman") {
                    gitStatusWrapper(
                        credentialsId: "github-access",
                        description: "Builds the container image",
                        failureDescription: "Container image failed to build",
                        successDescription: "Container image was successfully built",
                        gitHubContext: "build-container-image"
                    ) {
                        sh "podman build -t $image_name -f $implementation/Dockerfile $implementation"
                    }
                }
            }
        }
        stage("Upload Container Image") {
            steps {
                container("podman") {
                    gitStatusWrapper(
                        credentialsId: "github-access",
                        description: "Uploads the container image as $image_name",
                        failureDescription: "Could not upload the container image",
                        successDescription: "Container image was uploaded as $image_name",
                        gitHubContext: "upload-container-image"
                    ) {
                        milestone(ordinal: 100)
                        withCredentials([usernamePassword(credentialsId: 'registry-credentials', passwordVariable: 'registry_password', usernameVariable: 'registry_username')]) {
                            sh "podman login registry.finn-thorben.me -u $registry_username -p $registry_password"
                        }
                        sh "podman push $image_name"
                    }
                }
            }
        }
    }
}

