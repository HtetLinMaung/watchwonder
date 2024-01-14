pipeline {
    agent any

    stages {
        stage('Checkout') {
            steps {
                git 'https://github.com/HtetLinMaung/watchwonder.git'
            }
        }
        stage('Build Docker Image') {
            steps {
                script {
                    app = docker.build("htetlinmaung/watchwonder:latest")
                }
            }
        }
        stage('Push to Docker Registry') {
            steps {
                script {
                    docker.withRegistry('https://registry.hub.docker.com', 'docker-registry-credentials') {
                        app.push("latest")
                    }
                }
            }
        }
        stage('Deploy to VM') {
            steps {
                sshagent(credentials: ['zcomvm-ssh-credential-id']) {
                    sh 'ssh hlm@150.95.82.125 "cd watchwonder && docker pull htetlinmaung/watchwonder && docker-compose up -d"'
                }
            }
        }
    }
}