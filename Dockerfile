FROM rust:1.61.0


# Install the required dependencies



# Create a new directory to work in
WORKDIR /usr/src/myapp

#COPY BINARIES
COPY target/debug/flashdb /usr/src/myapp/flashdb


# Run the application
CMD ["./flashdb"]