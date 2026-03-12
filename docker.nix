{
  dockerTools,
  website,
}:
dockerTools.buildLayeredImage {
  name = "website";
  tag = "latest";

  contents = [
    website
    (dockerTools.caCertificates)
  ];

  config = {
    Cmd = [ "/bin/website" ];
    WorkingDir = "/";
  };
}
