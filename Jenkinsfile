pipeline {
    agent any

    environment {
        IMAGE_NAME = 'kamachess'
        IMAGE_TAG = "${env.BUILD_NUMBER}"
        DEPLOY_HOST = credentials('deploy-host') ?: ''
        DEPLOY_USER = credentials('deploy-user') ?: 'ubuntu'
        DEPLOY_PATH = credentials('deploy-path') ?: '/opt/kamachess'
        SSH_KEY = credentials('deploy-ssh-key') ?: ''
    }

    options {
        buildDiscarder(logRotator(numToKeepStr: '10'))
        timeout(time: 30, unit: 'MINUTES')
        ansiColor('xterm')
    }

    stages {
        stage('Checkout') {
            steps {
                script {
                    echo "Checking out code from ${env.GIT_URL}"
                    checkout scm
                    sh 'git rev-parse HEAD > .git/commit-hash'
                }
            }
        }

        stage('Build') {
            steps {
                script {
                    echo "Building Rust application using Docker..."
                    sh '''
                        docker build \
                            --tag ${IMAGE_NAME}:${IMAGE_TAG} \
                            --tag ${IMAGE_NAME}:latest \
                            -f Dockerfile \
                            .
                    '''
                }
            }
        }

        stage('Test') {
            steps {
                script {
                    echo "Running unit and integration tests..."
                    sh '''
                        docker build \
                            --target builder \
                            --tag ${IMAGE_NAME}-test:${IMAGE_TAG} \
                            -f Dockerfile \
                            .
                    '''
                    sh '''
                        docker run --rm \
                            -v $(pwd):/workspace \
                            -w /workspace \
                            ${IMAGE_NAME}-test:${IMAGE_TAG} \
                            cargo test --all-features --verbose
                    '''
                }
            }
        }

        stage('Lint') {
            steps {
                script {
                    echo "Running Clippy linter..."
                    sh '''
                        docker run --rm \
                            -v $(pwd):/workspace \
                            -w /workspace \
                            rust:1.88-bookworm \
                            bash -c "
                                cargo clippy --all-features -- -D warnings || true
                                cargo fmt -- --check || true
                            "
                    '''
                }
            }
        }

        stage('Tag Image') {
            steps {
                script {
                    echo "Tagging Docker image..."
                    sh '''
                        if [ -n "${DOCKER_REGISTRY}" ]; then
                            docker tag ${IMAGE_NAME}:${IMAGE_TAG} ${DOCKER_REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}
                            docker tag ${IMAGE_NAME}:latest ${DOCKER_REGISTRY}/${IMAGE_NAME}:latest
                        fi
                    '''
                }
            }
        }

        stage('Push to Registry') {
            when {
                expression { env.DOCKER_REGISTRY != null && env.DOCKER_REGISTRY != '' }
            }
            steps {
                script {
                    echo "Pushing image to Docker registry..."
                    withCredentials([usernamePassword(credentialsId: 'docker-registry-creds', usernameVariable: 'DOCKER_USER', passwordVariable: 'DOCKER_PASS')]) {
                        sh '''
                            echo ${DOCKER_PASS} | docker login ${DOCKER_REGISTRY} -u ${DOCKER_USER} --password-stdin
                            docker push ${DOCKER_REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}
                            docker push ${DOCKER_REGISTRY}/${IMAGE_NAME}:latest
                        '''
                    }
                }
            }
        }

        stage('Deploy') {
            when {
                branch 'main'
            }
            steps {
                script {
                    echo "Deploying to production server..."
                    def isLocalDeploy = !env.DEPLOY_HOST || env.DEPLOY_HOST == '' || env.DEPLOY_HOST == 'localhost' || env.DEPLOY_HOST == '127.0.0.1'
                    
                    if (isLocalDeploy) {
                        echo "Deploying locally (same EC2 instance as Jenkins)..."
                        sh '''
                            if [ -d "${DEPLOY_PATH}" ]; then
                                cd ${DEPLOY_PATH}
                                git pull || true
                                docker-compose up -d --build bot
                                sleep 10
                                if docker-compose ps bot | grep -q "Up"; then
                                    echo "Local deployment successful!"
                                    docker-compose ps
                                else
                                    echo "Local deployment failed - service not running"
                                    docker-compose logs bot
                                    exit 1
                                fi
                            else
                                echo "Error: Deployment path ${DEPLOY_PATH} does not exist"
                                exit 1
                            fi
                        '''
                    } else {
                        echo "Deploying remotely to ${env.DEPLOY_HOST}..."
                        sh '''
                            chmod +x ./scripts/jenkins-deploy.sh
                            ./scripts/jenkins-deploy.sh \
                                ${DEPLOY_HOST} \
                                ${DEPLOY_USER} \
                                ${DEPLOY_PATH} \
                                ${IMAGE_NAME}:${IMAGE_TAG}
                        '''
                    }
                }
            }
        }
    }

    post {
        success {
            echo "Pipeline succeeded!"
            script {
                archiveArtifacts artifacts: 'target/release/kamachess', allowEmptyArchive: true
            }
        }
        failure {
            echo "Pipeline failed!"
        }
        always {
            echo "Cleaning up..."
            script {
                sh '''
                    docker image prune -f
                    docker system prune -f --volumes || true
                '''
            }
        }
    }
}
